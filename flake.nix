{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/release-23.11";
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
			commonBuildInputs = with pkgs; [
				xorg.libX11
				xorg.libXcursor
				xorg.libXrandr
				xorg.libXi
				fontconfig
				gtk3
			];
			commonNativeInputs = with pkgs; [
				pkg-config
				cmake
			];
			vulkanPath = ''LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ]}"'';
			xdgPath = "XDG_DATA_DIRS=$XDG_DATA_DIRS:$GSETTINGS_SCHEMAS_PATH";
			green-updater-wrapped = craneLib.buildPackage {
				src = craneLib.cleanCargoSource (craneLib.path ./.);

				buildInputs = commonBuildInputs;
				nativeBuildInputs = with pkgs; [
					wrapGAppsHook
				] ++ commonNativeInputs;
			}; in {
				devShell = pkgs.mkShell {
					nativeBuildInputs = with pkgs; [
						cargo
						clippy
						cargo-outdated
					] ++ commonNativeInputs;

					buildInputs = commonBuildInputs;

					shellHook = ''
						export ${vulkanPath}
						export ${xdgPath}
					'';
				};

				packages.default = pkgs.stdenvNoCC.mkDerivation {
					name = "green-updater";
					src = green-updater-wrapped;

					installPhase = ''
						mv bin/green-updater bin/.green-updater-wrapped
						mkdir -p $out
						mv bin $out/bin
						echo "#!/bin/sh" > $out/bin/green-updater
						echo "${vulkanPath} $out/bin/.green-updater-wrapped" >> $out/bin/green-updater
						chmod +x $out/bin/green-updater
					'';
				};
			}
		);
}
