{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    ...
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      libPath = with pkgs;
        lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
    in {
      devShell = with pkgs;
        mkShell {
          buildInputs = [
            xorg.libxcb
          ];
          LD_LIBRARY_PATH = libPath;
        };
      formatter = with pkgs; writeShellScriptBin "alejandra" "exec ${lib.getExe alejandra} .";
    });
}
