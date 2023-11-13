{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, nixpkgs-mozilla, }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import nixpkgs-mozilla)
          ];
        };

        toolchain =
          (pkgs.rustChannelOf {
            date = "2023-11-11";
            channel = "nightly";
            sha256 = "sha256-0d/UxN6sekF+iQtebQl6jj/AQiT18Uag3CKbsCxc1E0=";
          })
          .rust;

        nativeLibPath = with pkgs; lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr  
          glib
          gsettings-desktop-schemas
        ];
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
            # Rust deps
            toolchain
            rust-analyzer
            pre-commit

            pkg-config

            # Audio
            alsa-lib

            # Controller support
            udev

            # Graphics (generic)
            vulkan-loader
            xorg.libxcb

            # GTK
            gtk3-x11.dev

            # Trunk for WASM serving
            trunk

            # Python needed for Py03
            python3

            # Gtk needed for GSettings Schemas
            gtk3
          ]);

          LD_LIBRARY_PATH = nativeLibPath;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          # Without the shellHook, there's no access to the GSettings Schemas
          shellHook = ''
            export XDG_DATA_DIRS=$GSETTINGS_SCHEMAS_PATH:${hicolor-icon-theme}/share:${gnome3.adwaita-icon-theme}/share
          '';
        };
      });
}
