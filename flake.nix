{
  description = "A dotfile manager";

  inputs.nixpkgs.url = "github:nixos/nixpkgs";

  outputs = {
    self,
    nixpkgs,
  }: let
    peridot = system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in
      pkgs.rustPlatform.buildRustPackage {
        pname = "peridot";
        version = "0.1.1";
        src = ./.;
        cargoSha256 = "0dkyja8i7dkxn1pr9xpxlxfwx0sffn6xnwv9nylrscapvqmz68vf";

        meta = with pkgs.lib; {
          description = "A dotfile manager";
          homepage = "https://github.com/whonore/peridot";
          license = with licenses; [mit];
          maintainers = with maintainers; [whonore];
        };
      };
  in {
    packages.aarch64-darwin.default = peridot "aarch64-darwin";
    packages.aarch64-linux.default = peridot "aarch64-linux";
    packages.i686-linux.default = peridot "i686-linux";
    packages.x86_64-darwin.default = peridot "x86_64-darwin";
    packages.x86_64-linux.default = peridot "x86_64-linux";
  };
}
