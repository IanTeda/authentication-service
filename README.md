<!-- Improved compatibility of back to top link -->
<a name="readme-top"></a>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

<!-- PROJECT HEADER -->
<br />
<div align="center">
    <a href="https://github.com/IanTeda/rust_starter">
        <img src="docs/images/default_rust_logo.png" alt="Logo" height="80">
    </a>
    <h3 align="center">Rust Starter Template</h3>
    <p align="center">
        A template repository for starting your next Rust project.
    <br />
        <a href="https://ianteda.github.io/rust_starter/"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    ·
    <a href="https://ianteda.github.io/rust_starter/issues">Report Bug</a>
    ·
    <a href="https://ianteda.github.io/rust_starter/issues">Request Feature</a>
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

This Repository is forked from and builds on [jeremychone-channel/rust-base](https://github.com/jeremychone-channel/rust-base). I have put it together as I learned my way through Rust and forms a good starting point for what most Rust projects required.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- PROJECT IS BUILT WITH -->
### In Built Features

The Rust Starter repository has the following features:

* Error handling best practice
* Cargo make starter
* Documentation build
* Unit and integration code snippets
* Rust development container
* Github CI actions
* VScode Rust extensions and settings

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- GETTING STARTED -->
## Getting Started

The **Getting Started** section contains prerequisites, installation and usage.

### Prerequisites

As a pre-request to using this Rust starter repository you will need:

1. Install protocol Buffer [compiler](https://grpc.io/docs/protoc-installation/?utm_source=fullstackwriter&utm_medium=fullstackwriter-blog)

```shell
apt install -y protobuf-compiler
```

### Installation

This repository is set up as a [template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) to let users generate new repositories with the same directory
structure and files. 

The easiest way to use this repository to get started is to:

1. Navigate to Github repository [https://github.com/IanTeda/rust_starter](https://github.com/IanTeda/rust_starter)
2. Click **Use this template** button up the top right
3. Select **Create a new repository** from the drop down menu


<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- USAGE -->
## Usage

The following Cargo Make tasks are provided as a starter:

* Build 

```zsh
grpcurl -plaintext\
  -d '{"name": "Tonic"}'\
  '[::1]:50051' helloworld.Greeter/SayHello
```

```zsh
grpcurl -plaintext\
  '[::1]:50051' ping.Ping/ping
```


GRPCUI

```zsh
grpcui --plaintext \
  -proto ./proto/ledger.proto \
 '127.0.0.1:8091'
```


<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ROADMAP -->
## Roadmap

Add features as I think they have a place in the starter repository

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- CONTACT -->
## Contact

Your Name - [@ian_teda](https://twitter.com/ian_teda) - [ian@teda.id.au](mailto:ian@teda.id.au)

Project Link: [https://github.com/IanTeda/personal_ledger](https://github.com/IanTeda/rust_starter)

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
[contributors-shield]: https://img.shields.io/github/contributors/IanTeda/personal_ledger.svg?style=for-the-badge
[contributors-url]: https://github.com/IanTeda/personal_ledger/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/IanTeda/personal_ledger.svg?style=for-the-badge
[forks-url]: https://github.com/IanTeda/personal_ledger/network/members
[stars-shield]: https://img.shields.io/github/stars/IanTeda/personal_ledger.svg?style=for-the-badge
[stars-url]: https://github.com/IanTeda/personal_ledger/stargazers
[issues-shield]: https://img.shields.io/github/issues/IanTeda/personal_ledger.svg?style=for-the-badge
[issues-url]: https://github.com/IanTeda/personal_ledger/issues
[license-shield]: https://img.shields.io/github/license/IanTeda/personal_ledger.svg?style=for-the-badge
[license-url]: https://github.com/IanTeda/personal_ledger/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/ianteda
[product-screenshot]: docs/images/screenshot.png

