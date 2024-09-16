import * as pulumi from "@pulumi/pulumi"
import * as awsx from "@pulumi/awsx"
import * as aws from "@pulumi/aws"

const config = new pulumi.Config()
const domain = config.require("domain")
const certificateArn = config.require("certificate_arn")

const repo = new awsx.ecr.Repository("image", {
  forceDelete: true,
})

const image = new awsx.ecr.Image("image", {
  repositoryUrl: repo.url,
  context: "./",
  platform: "linux/amd64",
  args: {
    PULUMI_ACCESS_TOKEN: config.require("PULUMI_ACCESS_TOKEN"),
    AWS_ACCESS_KEY_ID: config.require("AWS_ACCESS_KEY_ID"),
    AWS_SECRET_ACCESS_KEY: config.require("AWS_SECRET_ACCESS_KEY"),
  },
})

const cluster = new aws.ecs.Cluster("cluster")

const loadBalancer = new awsx.lb.ApplicationLoadBalancer("lb", {
  defaultTargetGroup: {
    port: 8080,
    protocol: "HTTPS",
  },
  listener: {
    port: 8080,
    certificateArn: certificateArn,
    protocol: "HTTPS",
    sslPolicy: "ELBSecurityPolicy-TLS13-1-2-2021-06",
  },
})

const service = new awsx.ecs.FargateService("service", {
  cluster: cluster.arn,
  assignPublicIp: true,
  taskDefinitionArgs: {
    container: {
      name: "tailcall-launcher-service",
      image: image.imageUri,
      cpu: 256,
      memory: 256,
      essential: true,
      portMappings: [
        {
          containerPort: 8080,
          hostPort: 8080,
          targetGroup: loadBalancer.defaultTargetGroup,
        },
      ],
    },
  },
})

export const url = pulumi.interpolate`http://${domain}:8080`
