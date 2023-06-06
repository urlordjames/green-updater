{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/release-23.05";
		flake-utils.url = "github:numtide/flake-utils";

		crane = {
			url = "github:ipetkov/crane";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = { self, nixpkgs, flake-utils, crane }:
		flake-utils.lib.eachDefaultSystem (system:
			let pkgs = import nixpkgs {
				inherit system;
			};
			craneLib = crane.lib.${system};
			green-updater = craneLib.buildPackage {
				src = craneLib.cleanCargoSource (craneLib.path ./.);

				nativeBuildInputs = with pkgs; [
					pkg-config
					wrapGAppsHook
				];

				buildInputs = with pkgs; [ gtk4 ];
			}; in {
				devShell = pkgs.mkShell {
					nativeBuildInputs = with pkgs; [
						pkg-config
						cargo
						clippy
					];

					buildInputs = with pkgs; [ gtk4 ];

					shellHook = ''
						XDG_DATA_DIRS=$XDG_DATA_DIRS:$GSETTINGS_SCHEMAS_PATH
					'';
				};

				packages.default = green-updater;
			}
		);
}
