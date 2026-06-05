# Open Sesame!
> Utter the magic phrase and open the cave anywhere!

A repertoire of stealth-oriented gateway server authenticators designed to minimise exposure of internal services to the public internet.


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
curl -X POST http://localhost:8009/knock   -H "Content-Type: application/json"   -d '{"passphrase":"test123"}'
```

And to access the internal dashboard
```
curl -v http://localhost:8009/dashboard
```

TO BE UPDATED










