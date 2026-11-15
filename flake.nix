{
  description = "Development environment for the TPTP Parser Arena";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rust = pkgs.rust-bin.stable."1.100.0".default.override {
            extensions = [ "rustfmt" "clippy" ];
          };
          sbtPackage = pkgs.sbt.override { jre = pkgs.jdk25; };
        in
        {
          default = pkgs.mkShell {
            packages = [
              rust
              pkgs.jdk25
              pkgs.maven
              sbtPackage
              pkgs.cmake
              pkgs.gcc
              pkgs.gnumake
              pkgs.pkg-config
              pkgs.git
              pkgs.nixpkgs-fmt
            ];

            JAVA_HOME = "${pkgs.jdk25}";

            shellHook = ''
              printf '%s\n' "TPTP Parser Arena development environment"
              printf 'Rust: %s\n' "$(rustc --version)"
              printf 'Java: %s\n' "$(java -version 2>&1 | head -n 1)"
            '';
          };
        });

      formatter = forAllSystems (system: (import nixpkgs { inherit system; }).nixpkgs-fmt);
    };
}
