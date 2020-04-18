# rust-opensc-sys
Safe bindings of [OpenSC](https://github.com/OpenSC/OpenSC) for the Rust programming language.

# Usage

Currently these bindings are work in progress and not in a usable state.

OpenSC is designed to provide a PKCS#11 API and be used as middleware through a set of tools that come with it. This is also why installing OpenSC doesn't install development headers. Using these tools in a script-like manner also satisfies the terms of the LGPL. This crate is for when the tools aren't versatile enough and you want to interface directly with the OpenSC library from Rust, e. g. for talking to a only partially ISO/IEC 7816 conformant card.

The OpenSC shared library must be installed on you system to build the crate and run the resulting binary.

# Licensing

Please note that OpenSC is LGPL-2.1 licensed. This crate will link dynamically against OpenSC to ensure the requirements of the LGPL are satisfied. The crate itself is licensed as MIT/Apache.
