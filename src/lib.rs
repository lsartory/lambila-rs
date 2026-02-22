pub mod vhdl_constant;
pub mod vhdl_entity;
pub mod vhdl_generic;
pub mod vhdl_port;
pub mod vhdl_project;
pub mod vhdl_range;
pub mod vhdl_signal;
pub mod vhdl_token;
pub mod vhdl_type;

pub use vhdl_project::VhdlProject;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vhdl_project_hierarchy() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity AND_Gate is
    Port ( A : in  STD_LOGIC;
           B : in  STD_LOGIC;
           Y : out STD_LOGIC);
end AND_Gate;

entity OR_Gate is
    Port ( A : in  STD_LOGIC;
           B : in  STD_LOGIC;
           Y : out STD_LOGIC);
end OR_Gate;

entity Top_Level is
    Port ( In1 : in STD_LOGIC;
           In2 : in STD_LOGIC;
           Out1 : out STD_LOGIC);
end Top_Level;

architecture Structural of Top_Level is
begin
    inst_and: entity work.AND_Gate
        port map (A => In1, B => In2, Y => Out1);

    inst_or: OR_Gate generic map (delay => 5ns) port map (A => In1, B => In2, Y => Out1);
end Structural;
";
        let mut project = VhdlProject::new();
        project
            .parse_reader(vhdl_content.as_bytes(), "test.vhd")
            .unwrap();

        assert_eq!(project.entities.len(), 3);

        let and_gate = project
            .entities
            .iter()
            .find(|e| e.name == "AND_Gate")
            .unwrap();
        assert_eq!(and_gate.ports.len(), 3);

        let top_level = project
            .entities
            .iter()
            .find(|e| e.name == "Top_Level")
            .unwrap();
        assert_eq!(top_level.architectures.len(), 1);
        assert_eq!(top_level.architectures[0].children.len(), 2);
        assert!(
            top_level.architectures[0]
                .children
                .iter()
                .any(|c| c.name == "AND_Gate")
        );
        assert!(
            top_level.architectures[0]
                .children
                .iter()
                .any(|c| c.name == "OR_Gate")
        );
    }

    #[test]
    fn test_duplicate_instantiations() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity Basic_Gate is
    Port ( A : in  STD_LOGIC;
           Y : out STD_LOGIC);
end Basic_Gate;

entity Top_Level is
    Port ( In1 : in STD_LOGIC;
           Out1 : out STD_LOGIC);
end Top_Level;

architecture Structural of Top_Level is
begin
    inst_one: entity work.Basic_Gate port map (A => In1, Y => Out1);
    inst_two: entity work.Basic_Gate port map (A => In1, Y => Out1);

    gen_gates: for i in 0 to 1 generate
        inst_gen: entity work.Basic_Gate port map (A => In1, Y => Out1);
    end generate;
end Structural;
";
        let mut project = VhdlProject::new();
        project
            .parse_reader(vhdl_content.as_bytes(), "test.vhd")
            .unwrap();

        let top_level = project
            .entities
            .iter()
            .find(|e| e.name == "Top_Level")
            .unwrap();

        assert_eq!(top_level.architectures[0].children.len(), 4);
        assert_eq!(
            top_level.architectures[0]
                .children
                .iter()
                .filter(|c| c.name == "Basic_Gate")
                .count(),
            4
        );
    }

    #[test]
    fn test_range_generate() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity Basic_Gate is
    Port ( A : in  STD_LOGIC;
           Y : out STD_LOGIC);
end Basic_Gate;

entity Top_Level is
    Port ( Ptr_Array : in STD_LOGIC_VECTOR(7 downto 0);
           Out1 : out STD_LOGIC);
end Top_Level;

architecture Structural of Top_Level is
begin
    gen_gates: for i in 0 to 7 generate
        inst_gen: entity work.Basic_Gate port map (A => Ptr_Array(i), Y => Out1);
    end generate;
end Structural;
";
        let mut project = VhdlProject::new();
        project
            .parse_reader(vhdl_content.as_bytes(), "test.vhd")
            .unwrap();

        let top_level = project
            .entities
            .iter()
            .find(|e| e.name == "Top_Level")
            .unwrap();

        assert_eq!(top_level.architectures[0].children.len(), 8);
        assert_eq!(
            top_level.architectures[0]
                .children
                .iter()
                .filter(|c| c.name == "Basic_Gate")
                .count(),
            8
        );
    }

    #[test]
    fn test_generics_parsing() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity MyEntity is
    generic (
        WIDTH : integer := 8;
        DEPTH : integer := 16
    );
    port (
        CLK : in std_logic;
        DATA : in std_logic_vector(WIDTH-1 downto 0)
    );
end MyEntity;
";
        let mut project = VhdlProject::new();
        project
            .parse_reader(vhdl_content.as_bytes(), "test.vhd")
            .unwrap();

        assert_eq!(project.entities.len(), 1);
        let entity = &project.entities[0];
        assert_eq!(entity.name, "MyEntity");
        assert_eq!(entity.generics.len(), 2);
        assert_eq!(entity.generics[0].name, "WIDTH");
        assert_eq!(entity.generics[0].default_value, Some("8".to_string()));
        assert_eq!(entity.generics[1].name, "DEPTH");
        assert_eq!(entity.generics[1].default_value, Some("16".to_string()));
        assert_eq!(entity.ports.len(), 2);
    }
}
