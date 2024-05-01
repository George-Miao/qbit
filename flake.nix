{
  description = "QBit flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    rust-overlay,
    nixpkgs,
    ...
  }: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ] (system:
        function (import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        }));
  in {
    devShells = forAllSystems (
      pkgs:
        with pkgs; let
          llvm = pkgs.llvmPackages_latest;
          packages =
            [
              llvm.bintools
              llvm.libstdcxxClang
              openssl
              pkg-config
            ]
            ++ (
              if pkgs.system == "aarch64-darwin" || pkgs.system == "x86_64-darwin"
              then [
                darwin.apple_sdk.frameworks.SystemConfiguration
              ]
              else []
            );
        in {
          stable = mkShell.override {stdenv = stdenvNoLibs;} {
            packages =
              [
                (rust-bin.stable.latest.default.override
                  {
                    extensions = ["rust-src"];
                  })
              ]
              ++ packages;
          };
          default = mkShell.override {stdenv = stdenvNoLibs;} {
            packages =
              [
                (
                  rust-bin.selectLatestNightlyWith (toolchain:
                    toolchain.default.override {
                      extensions = ["rust-src"];
                    })
                )
              ]
              ++ packages;
          };
        }
    );
  };
}
