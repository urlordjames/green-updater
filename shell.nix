{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
	nativeBuildInputs = with pkgs; [
		pkg-config
		cargo
		wrapGAppsHook
	];

	buildInputs = with pkgs; [
		gtk4
	];

	shellHook = ''
		XDG_DATA_DIRS=$XDG_DATA_DIRS:$GSETTINGS_SCHEMAS_PATH
	'';
}
