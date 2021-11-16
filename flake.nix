{
  description = "A dotfile manager";

  inputs.nixpkgs.url = "github:nixos/nixpkgs";

  outputs = { self, nixpkgs }:
    let pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in {
      defaultPackage.x86_64-linux = pkgs.rustPlatform.buildRustPackage {
        pname = "peridot";
        version = "0.1.1";
        src = ./.;
        cargoSha256 = "0dkyja8i7dkxn1pr9xpxlxfwx0sffn6xnwv9nylrscapvqmz68vf";

        meta = with pkgs.lib; {
          description = "A dotfile manager";
          homepage = "https://github.com/whonore/peridot";
          license = with licenses; [ mit ];
          maintainers = with maintainers; [ whonore ];
        };
      };
    };

}
