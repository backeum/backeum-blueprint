# Backeum blueprint
[![Test](https://github.com/backeum/donation-component/actions/workflows/test.yml/badge.svg)](https://github.com/backeum/donation-component/actions/workflows/test.yml) [![Security audit](https://github.com/backeum/donation-component/actions/workflows/audit.yml/badge.svg)](https://github.com/backeum/donation-component/actions/workflows/audit.yml) [![Compile](https://github.com/backeum/donation-component/actions/workflows/compile.yml/badge.svg)](https://github.com/backeum/donation-component/actions/workflows/compile.yml) ![Static Badge](https://img.shields.io/badge/Scrypto-v1.0.0-blue)


## Overview

Backeum Blueprint is the backbone of [Backeum](https://backeum.com), a platform connecting creators and backers through
cryptocurrency donations. The blueprint encapsulates the logic of smart contracts, meticulously crafted in Scrypto, and
is deployed to the Radix Distributed Ledger Technology (DLT).

## Components

### 1. Repository Component

The repository component is a factory component that delegates permissions to mint on the NFT resource. 
It is also responsible for merging different NFTs. The repository component is instantiated and managed by the Backeum
team. 

Responsible for:

- Owning the NFT minting process.
- Merging NFTs from the same collection.
- Serving as a factory that delegates permissions to mint on the NFT resource to the collection components.

### 2. Collection Component

Creators on [Backeum](https://backeum.com) use the collection component. Its primary functions include:

- Receiving donations on behalf of the creators.
- Issuing a trophy NFT as a token of appreciation and proof of backing a creator.

## Integration with Backeum Platform

This blueprint powers the smart contract interactions on the [Backeum platform](https://backeum.com), a hub where
creators and backers come together. Creators benefit from the support of the community, while backers receive unique
trophy NFTs, cementing their role in the creator's journey.

## Verify the integrity of the blueprint

The blueprint is compiled and deployed to the Radix DLT according to RDX Works recommendation of compiling the code
deterministically. Be sure to install [Docker](https://docs.docker.com/engine/install/) before proceeding.

Pull the official scrypto-builder image from Docker Hub.
```shell
DOCKER_DEFAULT_PLATFORM=linux/amd64 docker pull radixdlt/scrypto-builder:v1.0.0
```

Clone the repository
```shell
git clone git@github.com:backeum/backeum-blueprint.git
```

Navigate into the git repository and then, run
```shell
DOCKER_DEFAULT_PLATFORM=linux/amd64 docker run -v $(pwd):/src radixdlt/scrypto-builder:v1.0.0
```

This will compile the code and generate a `build` directory. The `build` directory contains the compiled code.

## Contribute

Contributions are always welcome! Whether it's enhancing the documentation, proposing new features, or fixing bugs, we
value your insights and contributions.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Support and Community

For support or to join the Backeum community:
- Visit our [Platform](https://backeum.com)
- Join our [Discord](https://discord.gg/m9MfMugGSn) or [Telegram](https://t.me/backeum) server.

---

Made with 💙 by the Backeum team.