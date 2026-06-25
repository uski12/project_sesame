## I used DuckDNS to get a dynamic domain for my private server for free. The below guide has only been written for DuckDNS.

Due to my ISP's CGNAT, IPv4 connections don't reach my internal server behind my router. Which is why I switched to IPv6. It worked out of the box for me, but may require a bit of setting up. YMMV.

The DuckDNS domain is unfortunately HTTP by default. To upgrade to HTTPS, you can use **Caddy** to autogenerate TLS certificates. Caddy automatically does Let's Encrypt challenges to get certificates for you. Search up Caddy for more information.

## IMPORTANT

If your router hides behind your ISP's CGNAT, it significantly complicates this process - Let's Encrypt won't be able to reach your server using IPv4 unless your ISP provisions a public IP for you/port forwards to your router (former unlikely in my country, and the latter VERY unlikely anywhere). And I couldn't find a way to force Let's Encrypt to use IPv6 at this time. You can also try reverse tunneling.

As an alternative, you can complete the challenge using DNS-01. It directly goes to your domain under DuckDNS and completes the challenge without ever contacting your private server, mitigating the IPv4 connectivity issue. Caddy automatically does this for a few other DDNS providers.

But, for DuckDNS, you must modify your caddy binary file to include the DuckDNS plugin module. This is done through xcaddy. A sample Caddyfile for use with the modified Caddy binary is given below 
For more information, please search up xcaddy and building it with the DuckDNS plugin.


Copy-paste instructions below:
```
go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest

# installed in this directory by default
cd ~/go/bin

# check if xcaddy was installed
./xcaddy version

./xcaddy build --with github.com/caddy-dns/duckdns

# verify if plugin was installed
./caddy list-modules | grep duckdns


```


#### /etc/caddy/Caddyfile
```
{
	email yourmail@email.com
}


your-domain.duckdns.org {
	tls {
		dns duckdns {DUCKDNS_TOKEN}
	}

	reverse_proxy 127.0.0.1:8009
}
```

Use the same email as your DuckDNS account, along with your domain name and DuckDNS account token.
