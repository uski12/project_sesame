<p align="center"><img src="https://github.com/user-attachments/assets/8c186907-8fad-4da5-9f5e-559e3572148b" alt="Logo"> </p>
<h3 align="center"> Open Sesame! </h3>
<p align="center"><sub><em>Utter the magic phrase and open the cave anywhere!</em></sub></p>


## Overview

> Internet -> Secret knock -> Server exposes itself temporarily -> Internal services

The end goal is to be able to access a private server on the internet from anywhere while protecting it against bad actors.  Varying levels of stealth (invisibility) available.

## General Architecture

> Internet -> Gateway -> Auth layer -> Internal dashboard

### Gateway
- Logs requests
- Validates incoming requests or drops them
- Access control
- Reverse proxying
- Minimise exposure

### Dashboard
Internal server, runs locally and never directly exposed to internet

## Usage

`TEMPORARY`

Running the gateway
```
cd ./https-sesame
cargo run
```

Running the internal server
```
cd ./server-sesame
uvicorn main:app --host 127.0.0.1 --port 3000
```

## Testing

`TEMPORARY`

In bash, to knock
```
curl -X POST http://localhost:8080/knock -H "Content-Type: application/json" -d "{\"passphrase\":\"test123\",\"nonce\":\"$(uuidgen)\",\"timestamp\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"}"
```
With caddy reverse proxying and DuckDNS,
```
curl -X POST https://domain.duckdns.org/knock -H 'Content-Type: application/json' -d "{\"passphrase\":\"test123\", \"nonce\": \"$(uuidgen)\", \"timestamp\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"}"
```



In Windows Powershell,
```
$body = [ordered]@{
    passphrase = "test123"
    nonce = "hi"
    timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
} | ConvertTo-Json -Compress

$body | Set-Content body.json -NoNewline

curl.exe -v -H "Content-Type: application/json" --data-binary "@body.json" http://localhost:8009/knock
```

And to access the internal dashboard
```
curl -v http://localhost:8009/dashboard
```


Converting to HTTPS:
```
SEE hosting/HOSTING.md for more information
```


TO BE UPDATED









