import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as origins from "aws-cdk-lib/aws-cloudfront-origins";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as targets from "aws-cdk-lib/aws-route53-targets";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as apigateway from "aws-cdk-lib/aws-apigateway";

export class AwsCdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const domain = process.env.DOMAIN || "";
    const acmId = process.env.ACM_ID || "";
    const hostedZoneId = process.env.HOSTED_ZONE_ID || "";
    const projectRoot = process.env.PROJECT_ROOT || "";

    const assetsBucket = new s3.Bucket(this, "Bucket", {
      bucketName: domain,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
    });

    const certificate = acm.Certificate.fromCertificateArn(
      this,
      "Certificate",
      acmId,
    );

    const func = new lambda.Function(this, "Function", {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      code: lambda.Code.fromAsset(projectRoot + "/dist"),
      handler: "bootstrap",
      environment: {
        NO_COLOR: "true",
        ASSETS_PATH: "./",
      },
      memorySize: 128,
    });

    const api = new apigateway.LambdaRestApi(this, "Api", {
      handler: func,
      proxy: true,
    });

    const s3Origin = new origins.S3Origin(assetsBucket);
    const apiOrigin = new origins.RestApiOrigin(api);

    const cf = new cloudfront.Distribution(this, "Distribution", {
      defaultBehavior: {
        origin: apiOrigin,
        cachePolicy: cloudfront.CachePolicy.CACHING_DISABLED,
        allowedMethods: cloudfront.AllowedMethods.ALLOW_ALL,
        cachedMethods: cloudfront.CachedMethods.CACHE_GET_HEAD_OPTIONS,
        originRequestPolicy:
          cloudfront.OriginRequestPolicy.ALL_VIEWER_EXCEPT_HOST_HEADER,
      },
      additionalBehaviors: {
        "/assets/*": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.jpg": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.avif": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.png": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.js": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.css": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.html": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.ico": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.svg": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/icons/*": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/images/*": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
      },
      domainNames: [domain],
      certificate,
    });

    const zone = route53.HostedZone.fromHostedZoneAttributes(
      this,
      "zone-attribute",
      {
        zoneName: domain,
        hostedZoneId,
      },
    );

    new route53.ARecord(this, "IpV4Record", {
      zone,
      target: route53.RecordTarget.fromAlias(new targets.CloudFrontTarget(cf)),
    });

    new route53.AaaaRecord(this, "IpV6Record", {
      zone,
      target: route53.RecordTarget.fromAlias(new targets.CloudFrontTarget(cf)),
    });
  }
}
