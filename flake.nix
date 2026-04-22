{
  description = "gitquarry - terminal CLI for explicit GitHub repository search";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        gitquarry = pkgs.callPackage ./nix/package.nix { };
      in
      {
        packages.default = gitquarry;

        apps.default = {
          type = "app";
          program = "${gitquarry}/bin/gitquarry";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            rustc
            rustfmt
          ];
        };
      }
    );
}
