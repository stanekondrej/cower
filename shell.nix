let
   nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-25.11";
   pkgs = import nixpkgs { config = {}; overlays = []; };
 in

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
	pkgs.clippy
	pkgs.openssl
	pkgs.wayland
  ];

  shellHook = ''
    export RUSTUP_HOME="$PWD/.rustup"
    export CARGO_HOME="$PWD/.cargo"
    mkdir -p "$RUSTUP_HOME" "$CARGO_HOME"
  '';
}

