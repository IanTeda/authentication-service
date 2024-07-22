<!-- Improved compatibility of back to top link -->
<a name="readme-top"></a>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Watchers][watchers-shield]][watchers-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<!-- PROJECT HEADER -->
<br />
<div align="center">
    <a href="https://github.com/IanTeda/authentication_microservice">
        <!--suppress CheckImageSize -->
<img src="docs/images/logo.png" alt="Logo" height="80">
    </a>
    <h3 align="center">Authentication Microservice</h3>
    <p align="center">
        Authentication is required for most application, by breaking authentication into a microservice I hope to reuse and develop authentication through its own workflow.
    <br />
    ·
    <a href="https://ianteda.github.io/authentication_microservice/issues">Report Bug</a>
    ·
    <a href="https://ianteda.github.io/authentication_microservice/issues">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
        <li><a href="#features">Features</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#references">References & Similar Projects</a></li>
  </ol>
</details>


<!-- ABOUT THE PROJECT -->

## About The Project

This repository aims to provide a reusable authentication microservice for building other applications along side.

Following initial authentication of a user through a unique email and password, [tokens](https://jwt.io/) are used to
verify the authenticity of a request and maintain sessions. A token secret (phrase) is used to encode the token, which
can be used by other microservices to decode access tokens to confirm authenticity of a request.

Acknowledging that general wisdom says one should not roll there own authentication, this intent of this microservice is
not to be internet facing.

This repository started as the basis for a [Personal Ledger](https://github.com/IanTeda/personal_ledger) (Money)
application and as motivation to learn [Rust](https://www.rust-lang.org/).


<!-- PROJECT IS BUILT WITH -->

### Built With

The following technology stack has been used in building this microservice:

* [Rust Language](https://www.rust-lang.org/) - Rust by the way.
* [Tonic](https://github.com/hyperium/tonic) - A rust implementation of gRPC.
* [Argon2](https://en.wikipedia.org/wiki/Argon2) - A key derivation function that was selected as the winner of the 2015
  Password Hashing Competition.
* [JSON Web Tokens](https://jwt.io/) - An open, industry standard RFC 7519 method for representing claims securely
  between two parties.
* [Sqlx](https://github.com/launchbadge/sqlx) - SQLx is an async, pure Rust† SQL crate featuring compile-time checked
  queries without a DSL.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- PROJECT FEATURES CURRENT AND FUTURE -->

### Features

This microservice has the following features, with a future road map of features un-ticked below:

- [x] User management
- [x] Encrypted password
- [x] Session management
- [x] Access and Refresh tokens
- [ ] Docker image
- [ ] Last logged in
- [ ] Rate limitations
- [ ] Two factor authentication
- [ ] User sign up (registration)
- [ ] Verify email address
- [ ] Forgotten password email recovery
- [ ] OAuth integration
- [ ] Support other database types

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->

## Getting Started

The **Getting Started** section contains prerequisites, installation and usage.

### Prerequisites

As a pre-request to using this microservice includes the following:

* Prerequisite 1

### Installation

This microservice is intended to be used within a docker container


<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- USAGE -->

## Usage

The microservice can be used by accessing it through the configured IP and port, utilising the defined proto files.

The endpoint reflections can be explored through [gRPCurl](https://github.com/fullstorydev/grpcurl)
or [gRPC UI](https://github.com/fullstorydev/grpcui)

gRPCurl:

```zsh
grpcurl -plaintext 127.0.0.1:8091 ping.Ping/ping
```

gRPC UI:

```zsh
grpcui -import-path . -proto ./proto/ledger.proto -plaintext 127.0.0.1:8091
```

```zsh
grpcui -plaintext 127.0.0.1:8091
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any
contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also
simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- LICENSE -->

## License

Distributed under the GPL-3.0 License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- CONTACT -->

## Contact

Ian Teda - [@ian_teda](https://twitter.com/ian_teda) - [ian@teda.id.au](mailto:ian@teda.id.au)

Project
Link: [https://github.com/IanTeda/authentication_microservice](https://github.com/IanTeda/authentication_microservice)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- REFERENCES -->

## References  Similar Projects

* [YouTube Video](https://www.youtube.com/watch?v=oxx7MmN4Ib0&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
* [jeremychone-channel/rust-base](https://github.com/jeremychone-channel/rust-base)
* [Blessed RS](https://blessed.rs/crates) - An unofficial guide to the Rust ecosystem
* [Building gRPC APIs with Rust](https://konghq.com/blog/engineering/building-grpc-apis-with-rust)
* [Build and Deploy a gRPC](https://www.koyeb.com/tutorials/build-and-deploy-a-grpc-web-app-using-rust-tonic-and-react)
* [Let's build a gRPC server and client in Rust with tonic](https://www.thorsten-hans.com/grpc-services-in-rust-with-tonic/)
* [bentwire/tonic-template](https://github.com/bentwire/tonic-template/blob/dubplate/build.rs)

<p align="right">(<a href="#readme-top">back to top</a>)</p>


Below is a list of similar applications:

* [Name](#)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/IanTeda/authentication_microservice.svg?style=for-the-badge

[contributors-url]: https://github.com/IanTeda/authentication_microservice/graphs/contributors

[forks-shield]: https://img.shields.io/github/forks/IanTeda/authentication_microservice.svg?style=for-the-badge

[forks-url]: https://github.com/IanTeda/authentication_microservice/network/members

[stars-shield]: https://img.shields.io/github/stars/IanTeda/authentication_microservice.svg?style=for-the-badge

[stars-url]: https://github.com/IanTeda/authentication_microservice/stargazers

[issues-shield]: https://img.shields.io/github/issues/IanTeda/authentication_microservice.svg?style=for-the-badge

[issues-url]: https://github.com/IanTeda/authentication_microservice/issues

[license-shield]: https://img.shields.io/github/license/IanTeda/authentication_microservice?style=for-the-badge

[license-url]: https://github.com/IanTeda/authentication_microservice/blob/master/LICENSE.txt

[watchers-url]: https://github.com/IanTeda/authentication_microservice/watchers

[watchers-shield]: https://img.shields.io/github/watchers/IanTeda/authentication_microservice?style=for-the-badge

[product-screenshot]: docs/images/logo.png

