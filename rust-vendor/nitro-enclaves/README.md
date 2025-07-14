# nitro-enclaves

The `nitro-enclaves` crate provides a Rust implementation of the [AWS Nitro Enclaves][aws-nitro-overview-page] Linux APIs.

[aws-nitro-overview-page]: https://aws.amazon.com/ec2/nitro/nitro-enclaves

## Nitro Enclaves API

On systems that enable AWS nitro enclaves, the Linux kernel provides a userspace API for the `/dev/nitro_enclaves` device. This crate implements this API in a flexible and type-safe high-level interface.
