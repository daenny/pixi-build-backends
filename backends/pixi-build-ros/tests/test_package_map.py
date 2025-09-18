from pathlib import Path
import tempfile
from pixi_build_backend.types.platform import Platform
from pixi_build_backend.types.project_model import ProjectModelV1

from pixi_build_ros.ros_generator import ROSGenerator
from pixi_build_ros.utils import load_package_map_data


def test_package_loading(test_data_dir: Path):
    """Load the package map with overwrites."""
    robostack_file = Path(__file__).parent.parent / "robostack.yaml"
    other_package_map = test_data_dir / "other_package_map.yaml"
    result = load_package_map_data([robostack_file, other_package_map])
    assert "new_package" in result
    assert result["new_package"]["conda"] == ["new-package"], "Should be added"
    assert result["alsa-oss"]["conda"] == ["other-alsa-oss"], "Should be overwritten"
    assert "zlib" in result, "Should still be present"


def test_generate_recipe_with_custom_ros(package_xmls: Path, test_data_dir: Path):
    """Test the generate_recipe function of ROSGenerator."""
    # Create a temporary directory to simulate the package directory
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)

        # Copy the test package.xml to the temp directory
        package_xml_source = package_xmls / "custom_ros.xml"
        package_xml_dest = temp_path / "package.xml"
        package_xml_dest.write_text(package_xml_source.read_text(encoding="utf-8"))

        # Create a minimal ProjectModelV1 instance
        model = ProjectModelV1()

        # Create config for ROS backend
        config = {
            "distro": "noetic",
            "noarch": False,
            "extra-package-mappings": [str(test_data_dir / "other_package_map.yaml")],
        }

        # Create host platform
        host_platform = Platform.current()

        # Create ROSGenerator instance
        generator = ROSGenerator()

        # Generate the recipe
        generated_recipe = generator.generate_recipe(
            model=model,
            config=config,
            manifest_path=str(temp_path),
            host_platform=host_platform,
        )

        # Verify the generated recipe has the expected requirements
        assert generated_recipe.recipe.package.name.get_concrete() == "ros-noetic-custom-ros"

        req_string = list(str(req) for req in generated_recipe.recipe.requirements.run)
        assert "ros-noetic-ros-package" in req_string
        assert "ros-noetic-ros-package-msgs" in req_string
