{
  description = "Hashira (柱) — MCP server scaffold: tool registry, response helpers, and rmcp boilerplate";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
      ...
    }:
    let
      system = "aarch64-darwin";
      pkgs = import nixpkgs { inherit system; };

      props = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      version = props.package.version;
      pname = "hashira";

      package = pkgs.rustPlatform.buildRustPackage {
        inherit pname version;
        src = pkgs.lib.cleanSource ./.;
        cargoLock.lockFile = ./Cargo.lock;
        doCheck = true;
        meta = {
          description = props.package.description;
          homepage = props.package.homepage;
          license = pkgs.lib.licenses.mit;
        };
      };
    in
    {
      packages.${system} = {
        hashira = package;
        default = package;
      };

      overlays.default = final: prev: {
        hashira = self.packages.${final.system}.default;
      };

      devShells.${system}.default = pkgs.mkShellNoCC {
        packages = [
          pkgs.rustc
          pkgs.cargo
          pkgs.rust-analyzer
        ];
      };

      formatter.${system} = pkgs.nixfmt-tree;
    };
}
