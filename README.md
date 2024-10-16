# Pokemon API

## How to run

### Generate RSA keys

```sh
openssl genrsa -out private_key.pem 2048
openssl rsa -pubout -in private_key.pem -out public_key.pem
```

### Download repo or docker-compose.yaml only

Download only docker-compose.yaml:

```sh
wget https://raw.githubusercontent.com/Pokedex-gamba/pokemon-api/refs/heads/master/docker-compose.yaml
```

Or clone the repo:

```sh
git clone https://github.com/Pokedex-gamba/pokemon-api.git
```

### Edit docker-compose.yaml

Make sure `/decoding_key` inside the container is mounted to `public_key.pem` on your machine or set in environment variable.\
Then just edit the `docker-compose.yaml` according to comments.

### Finally start it

```sh
docker compose up -d
```

## How to use

First you need to generate RS256 JWT token with the `private_key.pem` and then set at in `authorization` header.

If you enabled `DEBUG`, then you will get debug responses from all routes.\
It will also enable `/docs` endpoint so don't forget to check it out!
