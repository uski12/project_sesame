# Changelog - Open Sesame!

## 28/06/25

- Fixed IP in gateway logs showing as loopback only
- Cleaned up auth.rs
- All console logs now persist under https-sesame/logs/

## 25/06/25

- Added information for DuckDNS DDNS hosting - hosting/HOSTING.md
- Changed from using IPv4 to IPv6 - see hosting/HOSTING.md for why
- Upgraded from HTTP to HTTPS. Check hosting/HOSTING.md for more info
- Configured internal server firewall

## 16/06/25
- Added Nonces (replay protection)
- Started work on timestamp signatures & verification
- Setup DuckDNS for personal domain, need to do router port forwarding

## 09/06/26

- Finished failed-attempt-blacklist
- Converted all IP types from String to IpAddr type
- Reset fails on successful knock
- Cleanups implemented
- Implemented debug traces (info!, warn!, error!, debug!) from tracing, tracing_subscriber

## 06/06/26

- Started working on maximum number of failed auth attempts before block

## 05/06/26

- Started work on nonces and HMAC (added failed_ip, nonce lists in AppState)
- Added more .env attributes
- Added README.md, CHANGELOG.md and TODO.md

## Pre-05/06/26

- Created the project
- Buncha other stuff
- Added .env and .env.example files

### HTTPS-Gateway
- Replaced mutexes with RwLocks
- Added Axum request logger middleware
