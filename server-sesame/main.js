/**
 * STAR LOGIN ENGINE - CONTROLLER & EVENT LOOP
 * 
 * Manages form state, keyboard capture, cursor, hover interactions,
 * and page transitions.
 */

import { ParticleSystem, TextSampler, UIBoundarySampler } from './engine.js';

// Setup classes
const system = new ParticleSystem();
const sampler = new TextSampler();

// DOM References
const canvas = document.getElementById('login-canvas');
const ctx = canvas.getContext('2d');
const uiOverlay = document.getElementById('ui-overlay');
const loginForm = document.getElementById('login-form');
const usernameInput = document.getElementById('username-input');
const passwordInput = document.getElementById('password-input');
const successScreen = document.getElementById('success-screen');
const logoutBtn = document.getElementById('logout-btn');

// Layout references for particle positioning
const refs = {
  usernameInput: document.getElementById('username-input'),
  passwordInput: document.getElementById('password-input')
};

// Engine states
const STATES = {
  COSMOS: 'COSMOS',         // Only stars, no UI
  USERNAME: 'USERNAME',     // Typing username
  PASSWORD: 'PASSWORD',     // Typing password
  TRANSITION: 'TRANSITION', // Warp speed animation
  SUCCESS: 'SUCCESS'        // Logged in
};

let currentState = STATES.COSMOS;
let dpr = window.devicePixelRatio || 1;
let mouse = { x: null, y: null };
let time = 0;

let usernameChars = [];
let passwordChars = [];

// Track active targets to clean up on state changes
const activeTargetGroups = new Set();

function init() {
  resizeCanvas();
  system.init();
  
  // Set initial state
  setEngineState(STATES.COSMOS);
  
  // Focus username input immediately on page load
  setTimeout(() => {
    usernameInput.focus();
  }, 100);
  
  // Attach event listeners
  window.addEventListener('resize', resizeCanvas);
  
  // Mouse tracking
  window.addEventListener('mousemove', (e) => {
    mouse.x = e.clientX;
    mouse.y = e.clientY;
  });
  window.addEventListener('mouseout', () => {
    mouse.x = null;
    mouse.y = null;
  });
  
  // Global click listener to keep active input focused (no state changes)
  window.addEventListener('click', (e) => {
    if (successScreen.classList.contains('active')) return;
    
    // Focus the correct input depending on current state
    if (currentState === STATES.COSMOS || currentState === STATES.USERNAME) {
      usernameInput.focus();
    } else if (currentState === STATES.PASSWORD) {
      passwordInput.focus();
    }
  });

  // Global keydown listener to focus input on Cosmos keypress
  window.addEventListener('keydown', (e) => {
    if (successScreen.classList.contains('active')) return;
    
    if (currentState === STATES.COSMOS) {
      usernameInput.focus();
    }
  });

  // Form Input Listeners
  usernameInput.addEventListener('input', () => {
    // Typing the first character triggers transition from COSMOS to USERNAME state
    if (currentState === STATES.COSMOS && usernameInput.value.length > 0) {
      setEngineState(STATES.USERNAME);
    }
    updateUsernameText();
  });
  
  usernameInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (usernameInput.value.trim().length > 0) {
        setEngineState(STATES.PASSWORD);
      }
    }
  });
  
  passwordInput.addEventListener('input', () => {
    updatePasswordText();
  });
  
  passwordInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (passwordInput.value.length > 0) {
        triggerLogin();
      }
    }
  });

  // Form submit (from Enter System button click)
  loginForm.addEventListener('submit', (e) => {
    e.preventDefault();
    if (currentState === STATES.PASSWORD && passwordInput.value.length > 0) {
      triggerLogin();
    }
  });

  // Logout / Disconnect action
  logoutBtn.addEventListener('click', () => {
    logout();
  });
  
  // Start loop
  tick();
}

function resizeCanvas() {
  dpr = window.devicePixelRatio || 1;
  canvas.width = window.innerWidth * dpr;
  canvas.height = window.innerHeight * dpr;
  canvas.style.width = `${window.innerWidth}px`;
  canvas.style.height = `${window.innerHeight}px`;
  system.resize(window.innerWidth, window.innerHeight);
}

// State Machine Controller
function setEngineState(state) {
  currentState = state;
  
  // Update CSS state classes on the layout container
  uiOverlay.className = `state-${state.toLowerCase()}`;
  
  if (state === STATES.COSMOS) {
    // Clear all inputs
    usernameInput.value = '';
    passwordInput.value = '';
    usernameInput.blur();
    passwordInput.blur();
    
    // Clear all character lists
    usernameChars = [];
    passwordChars = [];
    
    // Clear all target groups
    clearAllTargetGroups();
    
  } else if (state === STATES.USERNAME) {
    usernameInput.focus();
    updateUsernameText();
    
  } else if (state === STATES.PASSWORD) {
    passwordInput.focus();
    updateUsernameText(); // Updates text to be dim
    updatePasswordText();
  } else if (state === STATES.SUCCESS) {
    startDashboard();
  }
}

// Update the particle representation of typed username
function updateUsernameText() {
  const val = usernameInput.value;
  
  if (val.length === 0) {
    // If user backspaces all characters, return back to COSMOS state
    if (currentState === STATES.USERNAME) {
      setEngineState(STATES.COSMOS);
    }
  }
  
  const shiftType = currentState === STATES.PASSWORD ? 'dim_red' : 'red';
  reconcileText(val, usernameChars, shiftType, refs.usernameInput, false);
}

// Update the particle representation of typed password
function updatePasswordText() {
  const val = passwordInput.value;
  reconcileText(val, passwordChars, 'purple', refs.passwordInput, true);
}

function clearAllTargetGroups() {
  for (let groupId of activeTargetGroups) {
    system.deallocateGroup(groupId);
  }
  activeTargetGroups.clear();
}

// Reconcile character arrays to only add/delete new letters and slide them
function reconcileText(val, charList, shiftType, inputElement, isPassword) {
  const displayChars = isPassword ? Array(val.length).fill('•') : val.split('');
  
  // 1. Reconcile character items
  while (charList.length > displayChars.length) {
    const removed = charList.pop();
    const rect = inputElement.getBoundingClientRect();
    const centerY = rect.top + rect.height / 2;
    system.deallocateGroup(removed.id, window.innerWidth / 2, centerY);
    activeTargetGroups.delete(removed.id);
  }
  
  for (let i = 0; i < displayChars.length; i++) {
    const char = displayChars[i];
    if (i < charList.length) {
      if (charList[i].char !== char) {
        system.deallocateGroup(charList[i].id);
        const sampleResult = sampler.sampleChar(char);
        charList[i] = {
          id: `char_${Math.random().toString(36).substring(2, 9)}`,
          char: char,
          width: sampleResult.width,
          points: sampleResult.points
        };
      }
    } else {
      const sampleResult = sampler.sampleChar(char);
      const newCharObj = {
        id: `char_${Math.random().toString(36).substring(2, 9)}`,
        char: char,
        width: sampleResult.width,
        points: sampleResult.points
      };
      charList.push(newCharObj);
      activeTargetGroups.add(newCharObj.id);
    }
  }
  
  // 2. Position characters (Centering layout with spacing)
  if (charList.length === 0) return;
  
  const rect = inputElement.getBoundingClientRect();
  const centerY = rect.top + rect.height / 2;
  const centerX = window.innerWidth / 2;
  const charSpacing = 4;
  
  let totalWidth = 0;
  for (let i = 0; i < charList.length; i++) {
    totalWidth += charList[i].width;
  }
  totalWidth += (charList.length - 1) * charSpacing;
  
  let currentX = centerX - totalWidth / 2;
  
  for (let i = 0; i < charList.length; i++) {
    const charItem = charList[i];
    const charCenterX = currentX + charItem.width / 2;
    
    system.allocateGroup(charItem.id, charItem.points, charCenterX, centerY, shiftType, 1.0);
    currentX += charItem.width + charSpacing;
  }
}

// Dashboard Monitoring Loops
let uptimeSeconds = 0;
let uptimeInterval = null;
let metricsInterval = null;

function startDashboard() {
  uptimeSeconds = 0;
  updateUptimeDisplay();
  clearInterval(uptimeInterval);
  uptimeInterval = setInterval(() => {
    uptimeSeconds++;
    updateUptimeDisplay();
  }, 1000);

  updateGauges();
  clearInterval(metricsInterval);
  metricsInterval = setInterval(() => {
    updateGauges();
  }, 2500);

  const iframe = document.getElementById('terminal-iframe');
  if (iframe) {
    iframe.src = 'http://localhost:7681/';
  }
}

function stopDashboard() {
  clearInterval(uptimeInterval);
  clearInterval(metricsInterval);
  uptimeSeconds = 0;
  
  const cpuCircle = document.getElementById('cpu-circle');
  const ramCircle = document.getElementById('ram-circle');
  if (cpuCircle) cpuCircle.style.strokeDashoffset = '251.2';
  if (ramCircle) ramCircle.style.strokeDashoffset = '251.2';
}

function updateUptimeDisplay() {
  const counter = document.getElementById('uptime-counter');
  if (!counter) return;
  
  const hrs = Math.floor(uptimeSeconds / 3600).toString().padStart(2, '0');
  const mins = Math.floor((uptimeSeconds % 3600) / 60).toString().padStart(2, '0');
  const secs = (uptimeSeconds % 60).toString().padStart(2, '0');
  
  counter.textContent = `${hrs}:${mins}:${secs}`;
}

function updateGauges() {
  const cpuVal = Math.floor(15 + Math.random() * 25);
  const ramVal = Math.floor(45 + Math.random() * 15);
  const latMs = Math.floor(Math.random() * 3);

  const cpuText = document.getElementById('cpu-value');
  const ramText = document.getElementById('ram-value');
  const cpuCircle = document.getElementById('cpu-circle');
  const ramCircle = document.getElementById('ram-circle');
  const latency = document.getElementById('latency-value');

  if (cpuText) cpuText.textContent = `${cpuVal}%`;
  if (ramText) ramText.textContent = `${ramVal}%`;
  if (latency) latency.textContent = latMs === 0 ? '<1ms' : `${latMs}ms`;

  const r = 40;
  const circ = 2 * Math.PI * r;

  if (cpuCircle) cpuCircle.style.strokeDashoffset = circ - (cpuVal / 100) * circ;
  if (ramCircle) ramCircle.style.strokeDashoffset = circ - (ramVal / 100) * circ;
}

// Trigger transition warp and showcase success
function triggerLogin() {
  setEngineState(STATES.TRANSITION);
  system.startWarp();
  
  // Wait for 3D warp animation to complete, then slide success screen
  setTimeout(() => {
    successScreen.classList.remove('hidden');
    requestAnimationFrame(() => {
      successScreen.classList.add('active');
      setEngineState(STATES.SUCCESS);
    });
  }, 1600);
}

function logout() {
  stopDashboard();
  successScreen.classList.remove('active');
  setTimeout(() => {
    successScreen.classList.add('hidden');
    system.resetFromWarp();
    setEngineState(STATES.COSMOS);
    
    // Autofocus username input to allow quick re-entry
    usernameInput.focus();
  }, 800);
}

// Core animation loop
function tick() {
  time++;
  
  // Clear screen
  ctx.fillStyle = '#000000';
  ctx.fillRect(0, 0, window.innerWidth, window.innerHeight);
  
  // If in Success mode, add a subtle central dark blue gradient overlay to the background
  if (currentState === STATES.SUCCESS) {
    const grad = ctx.createRadialGradient(
      window.innerWidth / 2, window.innerHeight / 2, 10,
      window.innerWidth / 2, window.innerHeight / 2, Math.max(window.innerWidth, window.innerHeight) * 0.7
    );
    grad.addColorStop(0, 'rgba(12, 24, 48, 0.4)');
    grad.addColorStop(1, '#000000');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, window.innerWidth, window.innerHeight);
  }
  
  // Update and draw particles
  ctx.save();
  ctx.scale(dpr, dpr);
  
  system.update(time, mouse.x, mouse.y);
  system.draw(ctx);
  
  ctx.restore();
  
  requestAnimationFrame(tick);
}

// Initialize when DOM and premium fonts are fully loaded
if (document.fonts) {
  document.fonts.ready.then(init);
} else {
  window.addEventListener('DOMContentLoaded', init);
}
