{
  description = "Ikamaki - Friendly Sushi Roll generating rotations for Splatoon";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      # Slint and winit load these with dlopen, so they have to be reachable at
      # runtime and linking against them is not enough.
      runtimeLibs =
        pkgs: with pkgs; [
          fontconfig
          freetype
          libGL
          libxkbcommon
          wayland
          libx11
          libxcb
          libxcursor
          libxi
          libxrandr
        ];
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "ikamaki";
          version = (pkgs.lib.importTOML ./Cargo.toml).package.version;

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./build.rs
              ./src
            ];
          };
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            makeWrapper
            patchelf
            pkg-config
          ];
          buildInputs = runtimeLibs pkgs;

          # The save dialog goes through the XDG desktop portal, which NixOS only
          # runs when a desktop enables it. zenity is rfd's fallback, so ship it
          # to get a file picker either way. patchelf has to run before the
          # wrapper moves the real binary.
          postFixup = ''
            patchelf --add-rpath ${pkgs.lib.makeLibraryPath (runtimeLibs pkgs)} $out/bin/ikamaki
            wrapProgram $out/bin/ikamaki --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.zenity ]}
          '';

          meta = {
            description = "Friendly Sushi Roll generating rotations for Splatoon";
            license = pkgs.lib.licenses.mit;
            mainProgram = "ikamaki";
            platforms = systems;
          };
        };
      });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/ikamaki";
        };
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            clippy
            pkg-config
            rust-analyzer
            rustc
            rustfmt
            zenity
          ];
          buildInputs = runtimeLibs pkgs;

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (runtimeLibs pkgs);
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
