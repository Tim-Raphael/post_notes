{
  description = "post-notes - building a cute digital garden";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      git-hooks,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Pin a single toolchain so cargo, clippy and rustfmt agree on the
        # edition 2024 the crate requires.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };

        # Everything needed to build, test and lint the crate.
        buildDeps = [ rustToolchain ];

        # Extra tools for the live-reloading preview workflow.
        previewDeps = with pkgs; [
          live-server
          cargo-watch
        ];

        pre-commit-check = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            rustfmt = {
              enable = true;
              packageOverrides = {
                cargo = rustToolchain;
                rustfmt = rustToolchain;
              };
            };
            clippy = {
              enable = true;
              # Block commits on lint warnings, not just errors.
              settings.denyWarnings = true;
              packageOverrides = {
                cargo = rustToolchain;
                clippy = rustToolchain;
              };
            };
          };
        };
      in
      {
        checks.pre-commit = pre-commit-check;

        devShells = {
          # Default shell installs the git hooks and provides the toolchain.
          default = pkgs.mkShell {
            buildInputs = buildDeps ++ pre-commit-check.enabledPackages;
            shellHook = pre-commit-check.shellHook;
          };

          # Preview shell rebuilds on change and serves the generated site.
          preview = pkgs.mkShell {
            buildInputs = buildDeps ++ previewDeps;
            shellHook = ''
              cargo-watch -x 'run -- -i tests/fixtures/notes' &
              live-server output/ --index --open alpha.html
            '';
          };
        };
      }
    );
}
