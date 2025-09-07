{
  description = "tray-tui: system tray in your terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      rec {

        packages.tray-tui =
          with pkgs;
          rustPlatform.buildRustPackage rec {
            pname = manifest.name;
            inherit (manifest) version;

            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [
              installShellFiles
            ];

            postInstall = ''
              installShellCompletion --cmd tray-tui \
                --bash <($out/bin/tray-tui --completions bash) \
                --zsh <($out/bin/tray-tui --completions zsh) \
                --fish <($out/bin/tray-tui --completions fish)
            '';

            passthru.updateScript = nix-update-script { };

            meta = {
              description = "System tray in your terminal";
              homepage = "https://github.com/Levizor/tray-tui";
              license = lib.licenses.mit;
              mainProgram = "tray-tui";
              maintainers = with lib.maintainers; [ Levizor ];
              platforms = lib.platforms.linux;
            };
          };

        defaultPackage = packages.tray-tui;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkgconf
            rust-analyzer
            rustc
            cargo
            cargo-edit
            git
          ];
          RUST_BACKTRACE = "1";
          CARGO_INCREMENTAL = "1";
        };
      }
    );

}
