# Changelog - Open Sesame!

## Pre-05/06/26

- Created the project
- Buncha other stuff
- Added .env and .env.example files

### HTTPS-Gateway
- Replaced mutexes with RwLocks
- Added Axum request logger middleware

## 05/06/26

- Started work on nonces and HMAC (added failed_ip, nonce lists in AppState)
- Added more .env attributes
- Added README.md, CHANGELOG.md and TODO.md


## 06/06/26

- Implemented maximum number of failed auth attempts before block
