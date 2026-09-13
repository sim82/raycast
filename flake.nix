{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
  let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in
  {
    devShells.x86_64-linux.default = pkgs.mkShell rec
    {

      nativeBuildInputs = [
        pkgs.pkg-config
      ];
      buildInputs = [
        pkgs.udev pkgs.alsa-lib pkgs.vulkan-loader
        pkgs.xorg.libX11 pkgs.xorg.libXcursor pkgs.xorg.libXi pkgs.xorg.libXrandr # To use the x11 feature
        pkgs.libxkbcommon pkgs.wayland # To use the wayland feature
        pkgs.SDL2 pkgs.SDL2_mixer
      ];
      env = {
        LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
      };
      # shellHook = ''
      #   export LD_LIBRARY_PATH=${lib.makeLibraryPath buildInputs }
      # '';
    };
  };
}
