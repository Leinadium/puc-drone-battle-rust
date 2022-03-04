# Drone Battle - PUBLIC BASE

Essa é a versão base da API para a Competição de IA do Departamento de Informática da PUC-Rio.

Foi disponibilizada versões base em várias linguagens. Essa é uma versão escrita em **Rust**, por Daniel Guimarães

## Requisitos:

* [Cargo (Rust)](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Instalação:

    $ git clone https://github.com/Leinadium/puc-drone-battle-rust
    $ cd puc-drone-battle-rust
    $ git checkout public_base
    $ cd drone-battle
    $ cargo build --release

## Utilização:

O primeiro argumento do executável é o path do `config.json`

    $ cd drone-battle
    $ cargo build --release

    # exemplo para windows
    $ target\release\puc-drone-battle-rust.exe ../../../config.json

    # exemplo para linux
    $ ./target/release/puc-drone-battle-rust ../../../config.json

A documentação das funções pode ser gerada através de `cargo doc --no-deps --open`

## Edição

Modifique a função `AI.think()` em `src/api/ai.rs` 