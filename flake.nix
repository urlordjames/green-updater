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
			green-updater = craneLib.buildPackage {
				src = craneLib.cleanCargoSource (craneLib.path ./.);

				buildInputs = commonBuildInputs;
				nativeBuildInputs = commonNativeInputs;
			}; in {
				devShell = pkgs.mkShell {
					nativeBuildInputs = with pkgs; [
						cargo
						clippy
					] ++ commonNativeInputs;

					buildInputs = commonBuildInputs;

					shellHook = "export ${vulkanPath}";
				};

				packages.default = pkgs.stdenvNoCC.mkDerivation {
					name = "green-updater-wrapped";
					src = green-updater;

					installPhase = ''
						mkdir -p $out
						mv bin $out/bin
						echo "#!/bin/sh" > $out/bin/green-updater-wrapped
						echo "${vulkanPath} $out/bin/green-updater" >> $out/bin/green-updater-wrapped
						chmod +x $out/bin/green-updater-wrapped
					'';
				};
			}
		);
}
