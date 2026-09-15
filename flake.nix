{
  description = "aozora-rs: 青空文庫テキストのパース・EPUB/XHTML変換ライブラリおよびGUI/CLI/WASMアプリケーション";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Linux向けGPUIランタイム共有ライブラリ
        linuxRuntimeLibs = with pkgs; [
          vulkan-loader
          libxkbcommon
          wayland
          libx11
          libxcursor
          libxrandr
          libxi
          libxcb
          openssl
          alsa-lib
          fontconfig
          freetype
        ];

        # プラットフォーム別ビルド依存
        platformNativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
          pkgs.pkg-config
        ];

        platformBuildInputs =
          pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux (linuxRuntimeLibs ++ [ pkgs.vulkan-headers ])
          ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            AppKit
            CoreGraphics
            Metal
            Security
            SystemConfiguration
          ]);

        # Common development tools
        devTools = with pkgs; [
          # Rust ツールチェイン
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer

          # WASM 開発ツール
          wasm-pack
          lld
          binaryen

          # EPUB 規格検証 & テストツール
          epubcheck
          nodejs
          unzip
          curl
        ];

        # ayame CLI パッケージ
        ayame-cli = pkgs.rustPlatform.buildRustPackage {
          pname = "ayame-cli";
          version = "0.6.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = platformNativeBuildInputs;
          buildInputs = platformBuildInputs;

          cargoBuildFlags = [ "-p" "ayame-cli" ];
          cargoTestFlags = [ "-p" "ayame-cli" ];

          meta = with pkgs.lib; {
            description = "青空文庫テキストをEPUB3に変換する超高速CLIツール";
            homepage = "https://github.com/kinoko0518/aozora-rs";
            license = licenses.mit;
            mainProgram = "ayame";
          };
        };

        # ayame-appパッケージ
        ayame-app = pkgs.rustPlatform.buildRustPackage {
          pname = "ayame-app";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = platformNativeBuildInputs ++ [ pkgs.makeWrapper ];
          buildInputs = platformBuildInputs;

          cargoBuildFlags = [ "-p" "ayame-app" ];
          # GUIアプリのためサンドボックステストはスキップ
          doCheck = false;

          postInstall = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
            wrapProgram $out/bin/ayame-app \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath linuxRuntimeLibs}"
          '';

          meta = with pkgs.lib; {
            description = "青空文庫テキスト変換デスクトップアプリケーション (GPUI)";
            homepage = "https://github.com/kinoko0518/aozora-rs";
            license = licenses.mit;
            mainProgram = "ayame-app";
          };
        };

        # qaツールパッケージ
        aozora-rs-qa = pkgs.rustPlatform.buildRustPackage {
          pname = "aozora-rs-qa";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = platformNativeBuildInputs;
          buildInputs = platformBuildInputs;

          cargoBuildFlags = [ "-p" "aozora-rs-qa" ];
          doCheck = false;

          meta = with pkgs.lib; {
            description = "青空文庫テキスト変換の品質保証・プロファイリングツール";
            homepage = "https://github.com/kinoko0518/aozora-rs";
            license = licenses.mit;
            mainProgram = "qa";
          };
        };

        # WASM パッケージ生成
        wasm-pkg = pkgs.stdenv.mkDerivation {
          pname = "aozora-rs-wasm-pkg";
          version = "0.1.0";
          src = ./.;

          nativeBuildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.wasm-pack
            pkgs.lld
            pkgs.binaryen
          ];

          buildPhase = ''
            export HOME=$(mktemp -d)
            wasm-pack build ./applications/wasm --target web --out-dir $out
          '';

          dontInstall = true;
        };

        # デモページ WASM ビルド更新スクリプト
        updateWasmScript = pkgs.writeShellScriptBin "update-demo-wasm" ''
          set -euo pipefail
          export PATH="${pkgs.lib.makeBinPath [ pkgs.wasm-pack pkgs.rustc pkgs.cargo pkgs.lld pkgs.binaryen ]}:$PATH"
          echo "Building WASM for demopage using Flake toolchain..."
          wasm-pack build ./applications/wasm --target web --out-dir ../demopage/pkg
          echo "Done! WASM package updated in applications/demopage/pkg."
        '';

      in
      {
        packages = {
          default = ayame-cli;
          ayame = ayame-cli;
          ayame-cli = ayame-cli;
          ayame-app = ayame-app;
          qa = aozora-rs-qa;
          wasm = wasm-pkg;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = ayame-cli;
            exePath = "/bin/ayame";
          };
          ayame = flake-utils.lib.mkApp {
            drv = ayame-cli;
            exePath = "/bin/ayame";
          };
          ayame-app = flake-utils.lib.mkApp {
            drv = ayame-app;
            exePath = "/bin/ayame-app";
          };
          qa = flake-utils.lib.mkApp {
            drv = aozora-rs-qa;
            exePath = "/bin/qa";
          };
          update-demo-wasm = flake-utils.lib.mkApp {
            drv = updateWasmScript;
            exePath = "/bin/update-demo-wasm";
          };
        };

        # 開発環境
        devShells.default = pkgs.mkShell {
          packages = devTools ++ platformNativeBuildInputs ++ platformBuildInputs;

          shellHook = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath linuxRuntimeLibs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          '';
        };

        # nix flake check 用
        checks = {
          inherit ayame-cli;
        };
      });
}
