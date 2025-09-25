{
  description = "Development environment for jd-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let

        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (final: prev: {
              jd-diff-patch = final.buildGoModule {
                pname = "jd";
                version = "2.3.0";
                src = final.fetchFromGitHub {
                  owner = "josephburnett";
                  repo = "jd";
                  rev = "v2.3.0";
                  hash = "sha256-eaNP7cSJ0IxfHLmPaNAw5MQzD41AiOIjVbAjQkU8uec=";
                };
                vendorHash = "sha256-7FrCoYEkQPqoGoD4EGq5tHMe+USmzBbgaHfHOnQRvd8=";
                sourceRoot = "source/v2";
                subPackages = [ "jd" ];

                proxyVendor = true;

              };
            })
          ];
        };

      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [

            jq
            jd-diff-patch # Tool for diffing and patching JSON files

          ];

          shellHook = ''

          '';
        };
      }
    );
}
