/**
 * STAR LOGIN ENGINE - CORE GRAPHICS & PHYSICS ENGINE
 * 
 * Implements high-performance particle behaviors:
 * 1. Twinkling & drifting background stars with elastic home anchors (Cosmos Mode).
 * 2. Spring-damper physics for UI binding.
 * 3. Offscreen canvas character sampling & caching.
 * 4. 3D perspective projection for the warp transition.
 */

// Global constant config
const CONFIG = {
  particleCount: 3500,
  freeDrag: 0.94, 
  boundDampingMin: 0.82,
  boundDampingMax: 0.88,
  springMin: 0.025,
  springMax: 0.045,
  mouseRadius: 130, 
  mouseRepelForce: 0.65, 
  defaultStarColor: { h: 210, s: 70, l: 90 } // Soft cosmic white-blue
};

export class Particle {
  constructor(x, y, startInvisible = false) {
    this.x = x;
    this.y = y;
    
    // Elastic home coordinates to pull stars back into place
    this.homeX = x;
    this.homeY = y;
    
    this.vx = (Math.random() - 0.5) * 0.2;
    this.vy = (Math.random() - 0.5) * 0.2;
    
    // Original physics characteristics
    this.originalSize = Math.random() * 1.1 + 0.4; // Stars are 0.4px to 1.5px
    this.size = this.originalSize;
    this.originalLuminance = Math.random() * 45 + 45; // 45% to 90%
    
    // Twinkle mechanics
    this.twinkleSpeed = 0.005 + Math.random() * 0.015;
    this.twinklePhase = Math.random() * Math.PI * 2;
    
    // Spring physics configuration
    this.spring = CONFIG.springMin + Math.random() * (CONFIG.springMax - CONFIG.springMin);
    this.damping = CONFIG.boundDampingMin + Math.random() * (CONFIG.boundDampingMax - CONFIG.boundDampingMin);
    
    // Target association
    this.state = 'FREE'; // 'FREE', 'BOUND', 'DISPERSING'
    this.targetX = null;
    this.targetY = null;
    this.targetId = null;
    this.dead = false;
    
    // Color states with individual soft variation
    this.originalHue = CONFIG.defaultStarColor.h + (Math.random() - 0.5) * 20;
    this.color = { h: this.originalHue, s: CONFIG.defaultStarColor.s, l: this.originalLuminance };
    
    this.originalAlpha = Math.random() * 0.7 + 0.2;
    this.alpha = startInvisible ? 0.0 : this.originalAlpha;
    
    // 3D coordinates for Warp Effect
    this.x3d = 0;
    this.y3d = 0;
    this.z3d = 0;
  }

  update(time, width, height, mouseX, mouseY, warpActive, warpSpeed) {
    if (warpActive) {
      // 3D Warp physics
      this.z3d -= warpSpeed;
      if (this.z3d <= 10) {
        this.z3d = 1000 + Math.random() * 200;
        this.x3d = (Math.random() - 0.5) * width * 5;
        this.y3d = (Math.random() - 0.5) * height * 5;
      }
      
      const centerX = width / 2;
      const centerY = height / 2;
      
      this.prevX = this.x;
      this.prevY = this.y;
      
      this.x = centerX + this.x3d / (this.z3d * 0.002);
      this.y = centerY + this.y3d / (this.z3d * 0.002);
      
      this.alpha = Math.min(1.0, (1200 - this.z3d) / 400);
      this.size = Math.min(3, 400 / this.z3d);
      return;
    }

    if (this.state === 'FREE') {
      // Fade in slowly if spawned new to fill spots
      if (this.alpha < this.originalAlpha) {
        this.alpha = Math.min(this.originalAlpha, this.alpha + 0.015);
      } else {
        // Twinkle luminance adjustment
        const twinkle = Math.sin(time * this.twinkleSpeed + this.twinklePhase);
        this.alpha = 0.3 + twinkle * 0.25;
      }
      
      // Return slowly to default color & size
      this.color.h += (this.originalHue - this.color.h) * 0.05;
      this.color.s += (CONFIG.defaultStarColor.s - this.color.s) * 0.05;
      this.color.l += (this.originalLuminance - this.color.l) * 0.05;
      this.size += (this.originalSize - this.size) * 0.05;
      
      // Gentle spring force back to home position
      const homeDx = this.homeX - this.x;
      const homeDy = this.homeY - this.y;
      this.vx += homeDx * 0.0012; 
      this.vy += homeDy * 0.0012;
      
      // Minimal random Brownian drift
      this.vx += (Math.random() - 0.5) * 0.008;
      this.vy += (Math.random() - 0.5) * 0.008;
      
      this.vx *= CONFIG.freeDrag;
      this.vy *= CONFIG.freeDrag;
      
      this.x += this.vx;
      this.y += this.vy;
      
      // Mouse repulsion
      if (mouseX !== null && mouseY !== null) {
        const dx = this.x - mouseX;
        const dy = this.y - mouseY;
        const distSq = dx * dx + dy * dy;
        const radiusSq = CONFIG.mouseRadius * CONFIG.mouseRadius;
        
        if (distSq < radiusSq) {
          const dist = Math.sqrt(distSq);
          if (dist > 0) {
            const force = (CONFIG.mouseRadius - dist) / CONFIG.mouseRadius;
            const push = force * CONFIG.mouseRepelForce;
            this.vx += (dx / dist) * push * 0.15;
            this.vy += (dy / dist) * push * 0.15;
          }
        }
      }
      
    } else if (this.state === 'BOUND') {
      // Spring-damper physics towards target coordinate
      const ax = (this.targetX - this.x) * this.spring;
      const ay = (this.targetY - this.y) * this.spring;
      
      this.vx = (this.vx + ax) * this.damping;
      this.vy = (this.vy + ay) * this.damping;
      
      this.x += this.vx;
      this.y += this.vy;
      
      // Slow and extremely gentle shimmer to eliminate rapid flickering
      this.alpha = 0.95 + Math.sin(time * 0.03 + this.twinklePhase) * 0.05;
      
      // Subtle mouse interaction (elastic stretch)
      if (mouseX !== null && mouseY !== null) {
        const dx = this.x - mouseX;
        const dy = this.y - mouseY;
        const distSq = dx * dx + dy * dy;
        const mouseRadiusBound = CONFIG.mouseRadius * 0.6;
        const radiusSq = mouseRadiusBound * mouseRadiusBound;
        
        if (distSq < radiusSq) {
          const dist = Math.sqrt(distSq);
          if (dist > 0) {
            const force = (mouseRadiusBound - dist) / mouseRadiusBound;
            this.x += (dx / dist) * force * 0.6;
            this.y += (dy / dist) * force * 0.6;
          }
        }
      }
      
    } else if (this.state === 'DISPERSING') {
      // Drift with current velocity while fading out
      this.vx *= CONFIG.freeDrag;
      this.vy *= CONFIG.freeDrag;
      this.x += this.vx;
      this.y += this.vy;
      
      // Return slowly to original size & color during dispersing fade
      this.color.h += (this.originalHue - this.color.h) * 0.05;
      this.size += (this.originalSize - this.size) * 0.05;
      
      this.alpha -= 0.02;
      if (this.alpha <= 0) {
        this.dead = true;
      }
    }
  }

  draw(ctx, warpActive) {
    if (warpActive) {
      ctx.strokeStyle = `hsla(${this.color.h}, ${this.color.s}%, ${this.color.l}%, ${this.alpha})`;
      ctx.lineWidth = this.size;
      ctx.beginPath();
      ctx.moveTo(this.prevX, this.prevY);
      ctx.lineTo(this.x, this.y);
      ctx.stroke();
    } else {
      ctx.fillStyle = `hsla(${this.color.h}, ${this.color.s}%, ${this.color.l}%, ${this.alpha})`;
      if (this.size > 1.2) {
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.size, 0, Math.PI * 2);
        ctx.fill();
      } else {
        ctx.fillRect(this.x - this.size, this.y - this.size, this.size * 2, this.size * 2);
      }
    }
  }
}

export class TextSampler {
  constructor() {
    this.canvas = document.createElement('canvas');
    this.ctx = this.canvas.getContext('2d', { willReadFrequently: true });
    this.cache = new Map();
    this.fontSize = 80;
    this.fontFamily = '"Space Grotesk", "Orbitron", sans-serif';
  }

  sampleChar(char) {
    if (this.cache.has(char)) {
      return this.cache.get(char);
    }
    
    this.ctx.font = `bold ${this.fontSize}px ${this.fontFamily}`;
    const metrics = this.ctx.measureText(char);
    const width = Math.max(10, Math.ceil(metrics.width));
    const height = this.fontSize + 30;
    
    this.canvas.width = width + 20;
    this.canvas.height = height;
    
    this.ctx.fillStyle = '#000000';
    this.ctx.fillRect(0, 0, width + 20, height);
    
    this.ctx.fillStyle = '#ffffff';
    this.ctx.font = `bold ${this.fontSize}px ${this.fontFamily}`;
    this.ctx.textBaseline = 'middle';
    this.ctx.textAlign = 'center';
    this.ctx.fillText(char, (width + 20) / 2, height / 2);
    
    const imgData = this.ctx.getImageData(0, 0, width + 20, height);
    const points = [];
    const step = 5;
    
    for (let y = 0; y < height; y += step) {
      for (let x = 0; x < width + 20; x += step) {
        const idx = (y * (width + 20) + x) * 4;
        if (imgData.data[idx] > 120) {
          points.push({
            x: x - (width + 20) / 2,
            y: y - height / 2
          });
        }
      }
    }
    
    const result = { points, width };
    this.cache.set(char, result);
    return result;
  }

  sample(text, isPassword = false) {
    const displayString = isPassword ? '•'.repeat(text.length) : text;
    const cacheKey = `${displayString}_${isPassword ? 'pw' : 'raw'}`;
    
    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey);
    }
    
    if (!displayString) return [];
    
    this.ctx.font = `bold ${this.fontSize}px ${this.fontFamily}`;
    const metrics = this.ctx.measureText(displayString);
    const width = Math.max(20, Math.ceil(metrics.width) + 20);
    const height = this.fontSize + 30;
    
    this.canvas.width = width;
    this.canvas.height = height;
    
    this.ctx.fillStyle = '#000000';
    this.ctx.fillRect(0, 0, width, height);
    
    this.ctx.fillStyle = '#ffffff';
    this.ctx.font = `bold ${this.fontSize}px ${this.fontFamily}`;
    this.ctx.textBaseline = 'middle';
    this.ctx.textAlign = 'left';
    this.ctx.fillText(displayString, 10, height / 2);
    
    const imgData = this.ctx.getImageData(0, 0, width, height);
    const points = [];
    const step = 5;
    
    for (let y = 0; y < height; y += step) {
      for (let x = 0; x < width; x += step) {
        const idx = (y * width + x) * 4;
        if (imgData.data[idx] > 120) {
          points.push({
            x: x - width / 2,
            y: y - height / 2
          });
        }
      }
    }
    
    this.cache.set(cacheKey, points);
    return points;
  }
}

export class UIBoundarySampler {
  static sampleLine(width, spacing = 5) {
    const points = [];
    const halfW = width / 2;
    for (let x = -halfW; x <= halfW; x += spacing) {
      points.push({ x, y: 0 });
    }
    return points;
  }

  static sampleRect(width, height, spacing = 6) {
    const points = [];
    const halfW = width / 2;
    const halfH = height / 2;
    
    for (let x = -halfW; x <= halfW; x += spacing) {
      points.push({ x, y: -halfH });
      points.push({ x, y: halfH });
    }
    for (let y = -halfH + spacing; y < halfH; y += spacing) {
      points.push({ x: -halfW, y });
      points.push({ x: halfW, y });
    }
    
    return points;
  }
}

export class ParticleSystem {
  constructor() {
    this.particles = [];
    this.boundMap = new Map(); // TargetId -> Particle
    this.width = window.innerWidth;
    this.height = window.innerHeight;
    this.warpActive = false;
    this.warpSpeed = 0;
  }

  init() {
    this.particles = [];
    this.boundMap.clear();
    
    for (let i = 0; i < CONFIG.particleCount; i++) {
      const x = Math.random() * this.width;
      const y = Math.random() * this.height;
      this.particles.push(new Particle(x, y));
    }
  }

  resize(w, h) {
    const prevW = this.width;
    const prevH = this.height;
    this.width = w;
    this.height = h;
    
    for (let p of this.particles) {
      if (p.state === 'FREE') {
        p.x = (p.x / prevW) * w;
        p.y = (p.y / prevH) * h;
      }
      p.homeX = (p.homeX / prevW) * w;
      p.homeY = (p.homeY / prevH) * h;
    }
  }

  update(time, mouseX, mouseY) {
    if (this.warpActive) {
      this.warpSpeed = Math.min(80, this.warpSpeed + 1.2);
    }
    
    for (let p of this.particles) {
      p.update(time, this.width, this.height, mouseX, mouseY, this.warpActive, this.warpSpeed);
    }
    
    // Prune dead dispersing particles
    this.particles = this.particles.filter(p => !p.dead);
  }

  draw(ctx) {
    ctx.save();
    ctx.globalCompositeOperation = 'lighter';
    
    for (let p of this.particles) {
      p.draw(ctx, this.warpActive);
    }
    ctx.restore();
  }

  // Binds the nearest free stars to a set of targets with custom shifts and bolding
  allocateGroup(groupId, points, centerX, centerY, shiftType, sizeMultiplier = 1.0) {
    let hueShift = 0;
    let sizeScale = 2.2; // Bold scale for active letters
    let satVal = 70; // Original star saturation
    let lumScale = 1.30; // Slightly brighter than original
    
    if (shiftType === 'red') {
      hueShift = 0; // Keeping original bg star colors
      satVal = 70;
      lumScale = 1.30; // Slightly brighter
    } else if (shiftType === 'dim_red') {
      hueShift = 0;
      sizeScale = 1.05; // Return to normal size when dimmed
      satVal = 70;
      lumScale = 0.45; // Dimmed
    } else if (shiftType === 'purple') {
      hueShift = 40; // Slight purple/violet shift overlay (+40)
      satVal = 80;
      lumScale = 1.30; // Slightly brighter
    }
    
    points.forEach((pt, index) => {
      const targetId = `${groupId}_${index}`;
      const absTargetX = centerX + pt.x;
      const absTargetY = centerY + pt.y;
      
      let p = this.boundMap.get(targetId);
      
      if (p) {
        p.targetX = absTargetX;
        p.targetY = absTargetY;
        p.color.h = (p.originalHue + hueShift) % 360;
        p.color.s = satVal;
        p.color.l = p.originalLuminance * lumScale;
        p.size = p.originalSize * sizeMultiplier * sizeScale;
      } else {
        let nearestP = null;
        let minDistSq = Infinity;
        
        for (let particle of this.particles) {
          if (particle.state === 'FREE') {
            const dx = particle.x - absTargetX;
            const dy = particle.y - absTargetY;
            const distSq = dx * dx + dy * dy;
            
            if (distSq < minDistSq) {
              minDistSq = distSq;
              nearestP = particle;
            }
          }
        }
        
        if (nearestP) {
          nearestP.state = 'BOUND';
          nearestP.targetX = absTargetX;
          nearestP.targetY = absTargetY;
          nearestP.targetId = targetId;
          nearestP.color.h = (nearestP.originalHue + hueShift) % 360;
          nearestP.color.s = satVal;
          nearestP.color.l = nearestP.originalLuminance * lumScale;
          nearestP.size = nearestP.originalSize * sizeMultiplier * sizeScale;
          
          this.boundMap.set(targetId, nearestP);
          
          // Repopulation logic: Spawn a new background star around the area
          // where the old star was bound. This fills empty spots in background.
          const spawnRadius = 160;
          const angle = Math.random() * Math.PI * 2;
          const dist = Math.random() * spawnRadius;
          const spawnX = absTargetX + Math.cos(angle) * dist;
          const spawnY = absTargetY + Math.sin(angle) * dist;
          const newFreeP = new Particle(spawnX, spawnY, true); // startInvisible = true
          this.particles.push(newFreeP);
        }
      }
    });
    
    // Prune excess particles if target count decreased
    let idx = points.length;
    while (this.boundMap.has(`${groupId}_${idx}`)) {
      this.releaseParticle(`${groupId}_${idx}`, centerX, centerY);
      idx++;
    }
  }

  deallocateGroup(groupId, centerX = this.width / 2, centerY = this.height / 2) {
    let idx = 0;
    while (this.boundMap.has(`${groupId}_${idx}`)) {
      this.releaseParticle(`${groupId}_${idx}`, centerX, centerY);
      idx++;
    }
  }

  releaseParticle(targetId, centerX, centerY) {
    const p = this.boundMap.get(targetId);
    if (p) {
      p.state = 'DISPERSING'; // Fade out and disperse instead of instantly FREE
      p.targetX = null;
      p.targetY = null;
      p.targetId = null;
      
      // Explosion/dispersion burst velocity away from the element center
      const angle = Math.atan2(p.y - centerY, p.x - centerX) + (Math.random() - 0.5) * 0.4;
      const speed = 1.2 + Math.random() * 2.8;
      
      p.vx = Math.cos(angle) * speed;
      p.vy = Math.sin(angle) * speed;
    }
    this.boundMap.delete(targetId);
  }

  // Start 3D Warp speed transition
  startWarp() {
    this.warpActive = true;
    this.warpSpeed = 0.5;
    
    this.boundMap.clear();
    
    this.particles.forEach(p => {
      p.state = 'FREE';
      p.targetX = null;
      p.targetY = null;
      
      const centerX = this.width / 2;
      const centerY = this.height / 2;
      
      p.z3d = Math.random() * 800 + 400; // Depth
      p.x3d = (p.x - centerX) * (p.z3d * 0.002);
      p.y3d = (p.y - centerY) * (p.z3d * 0.002);
    });
  }

  // Reset from warp back to cosmos
  resetFromWarp() {
    this.warpActive = false;
    this.warpSpeed = 0;
    this.init();
  }
}
