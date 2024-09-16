# Tailcall Launchpad

This service is responsible to deploy tailcall instances on the cloud.

## Setup Instructions

- Dependencies

  -

  * docker engine: https://docs.docker.com/engine/install
  * NodeJS (v22+): https://nodejs.org/en/download/package-manager
  * pulumi: https://www.pulumi.com/docs/install
  * docker: https://docs.docker.com/engine/install

- ## `environment` variables.
  ```bash
  export AWS_ACCESS_KEY_ID="GET IT FROM IAM"
  export AWS_SECRET_ACCESS_KEY="GET IT FROM IAM"
  export AWS_REGION="eu-central-1" # Any region of your preference
  export PULUMI_ACCESS_TOKEN="Go to pulumi website"
  ```
- ## Execute

  ```
  pulumi up \
   -c PULUMI_ACCESS_TOKEN=$PULUMI_ACCESS_TOKEN \
   -c AWS_REGION=$AWS_REGION \
   -c AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID \
   -c AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY
  ```

## Local Env

```
$ docker build -t tailcall/launchpad .
$ docker run -d \
  --name=launchpad \
  -p 9090:8080 \
  -e AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID \
  -e AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY \
  -e PULUMI_ACCESS_TOKEN=$PULUMI_ACCESS_TOKEN \
  -e AWS_REGION=$AWS_REGION \
  -e SERVER_PORT=8080 \
  --replace \
```
