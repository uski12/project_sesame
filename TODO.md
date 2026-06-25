# TODO - Open Sesame!

## Overall
- [ ] Makefile (one command to run everything)

## Authentication & Gateway
- [X] IP temp timeouts
- [X] Reset failures on success
- [X] Convert from IPv4 to IPv6
- [X] Convert from HTTP to HTTPS - woohoo! Major PITA to do
- [X] Nonce replay signatures
- [X] Timestamp validation 

- [ ] FIX IP in gateway logs - shows 127.0.0.1 only due to reverse proxying
- [ ] Output logs to logfile
- [ ] Refactor & cleanup auth.rs code
- [ ] HMAC implementation?
- [ ] Cleanup for IPs and Nonces
- [ ] Rate limiting

## Dashboard
- [ ] Impressive dashboard - expand upon later

## Future
- [ ] UDP SPA (client and server required)
- [ ] SYN packet modification
