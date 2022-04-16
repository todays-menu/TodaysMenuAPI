# TodaysMenuAPI

TodaysMenu backend API

## Development

Install [AWS SAM][SAM] first.

### Start Dev Server

Prepare docker image

```sh
cd docker
docker build -t build-rust-provided.al2 .
```

Build (in project work dir)

```sh
make
```

Run local api gateway

```sh
make localstack
make apigw
```

### Connect To Apollo Sandbox

After starting the dev server you can connect to
`http://localhost:8080/graphql` to open the Apollo
GraphQL sandbox portal.

### Deploy

```sh
MODE=release make
sam deploy
```

[SAM]: https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/index.html
