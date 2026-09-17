{
  description = "Flake using pyproject.toml metadata";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    bast3st-py.url = "github:marcel-scherzinger/bast3st-py";
    # This is necessary for "bast3st" being available as library
    bast3st-py.inputs.nixpkgs.follows = "nixpkgs";
    bast3st-py.inputs.flake-parts.follows = "flake-parts";
  };
  outputs = inputs @ {flake-parts, ...}: let
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [];
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];

      perSystem = {
        pkgs,
        system,
        ...
      }: let
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.just
            pkgs.gnumake
          ];
          shellHook = ''
          '';
        };

        formatter = pkgs.alejandra;

        packages.src-catchable-rs =
          pkgs.writers.writePython3Bin "src-catchable-rs" {
            libraries = [
              inputs.bast3st-py.packages.${system}.bast3st-lib
            ];
          }
          # python
          ''
            from bast3st.catchable import err
            import sys
            fallback = "src/catchable/generated.rs"
            outfile = sys.argv[-1] if sys.argv[-1].endswith(".rs") else fallback

            project_url = "https://marcel-scherzinger.github.io/bast3st-py"
            base_url = f"{project_url}/ref_caterr.html#bast3st.catchable.err."

            options = [
                f"{' ' * 8}/// See [`bast3st-py.catchable.err.{x.name}`]" +
                f"({base_url}{x.name})\n" +
                f"{' ' * 8}const {x.name} = {bin(x.value)};"
                for x in err._member_map_.values()
            ]
            data = """// GENERATED\nuse super::cerr;
            bitflags::bitflags! {
                impl cerr : u32 {
            """ + "\n".join(options) + "\n    }\n}"

            if sys.argv[-1] == '-':
                print(data)
            else:
                with open(outfile, "w") as f:
                    f.write(data)
          '';
      };
    };
}
