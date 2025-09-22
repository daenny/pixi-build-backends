use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    str::FromStr,
};

use pixi_build_types::{BinaryPackageSpecV1, PackageSpecV1, PathSpecV1, ProjectModelV1, SourcePackageSpecV1, TargetSelectorV1, TargetV1, TargetsV1};
use pyo3::{exceptions::PyValueError, prelude::*};
use rattler_conda_types::{ParseStrictness, Version, VersionSpec};
use serde::Deserialize;
use serde_json::from_str;

#[pyclass]
#[derive(Clone)]
pub struct PyProjectModelV1 {
    pub(crate) inner: ProjectModelV1,
}

#[pymethods]
impl PyProjectModelV1 {
    #[new]
    #[pyo3(signature = (name, version=None))]
    pub fn new(name: Option<String>, version: Option<String>) -> Self {
        PyProjectModelV1 {
            inner: ProjectModelV1 {
                name,
                version: version.map(|v| {
                    v.parse()
                        .unwrap_or_else(|_| Version::from_str(&v).expect("Invalid version"))
                }),
                targets: None,
                description: None,
                authors: None,
                license: None,
                license_file: None,
                readme: None,
                homepage: None,
                repository: None,
                documentation: None,
            },
        }
    }

    #[staticmethod]
    pub fn from_json(json: &str) -> PyResult<Self> {
        let project: ProjectModelV1 = from_str(json).map_err(|err| {
            PyErr::new::<PyValueError, _>(format!(
                "Failed to parse ProjectModelV1 from JSON: {err}"
            ))
        })?;

        Ok(PyProjectModelV1 { inner: project })
    }

    #[staticmethod]
    pub fn from_json_file(path: &str) -> PyResult<Self> {
        let content = fs::read_to_string(path).map_err(|err| {
            PyErr::new::<PyValueError, _>(format!(
                "Failed to read ProjectModelV1 JSON file '{path}': {err}"
            ))
        })?;

        Self::from_json(&content)
    }

    #[staticmethod]
    pub fn from_test_json(json: &str) -> PyResult<Self> {
        let test_model: TestProjectModel = from_str(json).map_err(|err| {
            PyErr::new::<PyValueError, _>(format!(
                "Failed to parse test ProjectModel from JSON: {err}"
            ))
        })?;

        let project = convert_test_model_to_project_model_v1(test_model);

        Ok(PyProjectModelV1 { inner: project })
    }

    #[staticmethod]
    pub fn from_test_json_file(path: &str) -> PyResult<Self> {
        let content = fs::read_to_string(path).map_err(|err| {
            PyErr::new::<PyValueError, _>(format!(
                "Failed to read test ProjectModel JSON file '{path}': {err}"
            ))
        })?;

        Self::from_test_json(&content)
    }

    #[getter]
    pub fn name(&self) -> Option<&String> {
        self.inner.name.as_ref()
    }

    #[getter]
    pub fn version(&self) -> Option<String> {
        self.inner.version.as_ref().map(|v| v.to_string())
    }

    #[getter]
    pub fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    #[getter]
    pub fn authors(&self) -> Option<Vec<String>> {
        self.inner.authors.clone()
    }

    #[getter]
    pub fn license(&self) -> Option<String> {
        self.inner.license.clone()
    }

    #[getter]
    pub fn license_file(&self) -> Option<String> {
        self.inner
            .license_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
    }

    #[getter]
    pub fn readme(&self) -> Option<String> {
        self.inner
            .readme
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
    }

    #[getter]
    pub fn homepage(&self) -> Option<String> {
        self.inner.homepage.as_ref().map(|u| u.to_string())
    }

    #[getter]
    pub fn repository(&self) -> Option<String> {
        self.inner.repository.as_ref().map(|u| u.to_string())
    }

    #[getter]
    pub fn documentation(&self) -> Option<String> {
        self.inner.documentation.as_ref().map(|u| u.to_string())
    }

    pub fn _debug_str(&self) -> String {
        format!("{:?}", self.inner)
    }
}

impl From<ProjectModelV1> for PyProjectModelV1 {
    fn from(model: ProjectModelV1) -> Self {
        PyProjectModelV1 { inner: model }
    }
}

impl From<&ProjectModelV1> for PyProjectModelV1 {
    fn from(model: &ProjectModelV1) -> Self {
        PyProjectModelV1 {
            inner: model.clone(),
        }
    }
}

impl From<PyProjectModelV1> for ProjectModelV1 {
    fn from(py_model: PyProjectModelV1) -> Self {
        py_model.inner
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TestProjectModel {
    name: String,
    version: String,
    description: Option<String>,
    authors: Option<Vec<String>>,
    license: Option<String>,
    license_file: Option<String>,
    readme: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    targets: TestTargets,
}

#[derive(Debug, Clone, Deserialize)]
struct TestTargets {
    default_target: TestTarget,
    #[serde(default)]
    targets: HashMap<TestTargetSelector, TestTarget>,
}

#[derive(Debug, Clone, Deserialize)]
struct TestTarget {
    #[serde(default)]
    host_dependencies: HashMap<String, TestPackageSpec>,
    #[serde(default)]
    build_dependencies: HashMap<String, TestPackageSpec>,
    #[serde(default)]
    run_dependencies: HashMap<String, TestPackageSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TestPackageSpec {
    Binary(TestBinaryPackageSpec),
    Source(TestSourcePackageSpec),
}

#[derive(Debug, Clone, Deserialize)]
struct TestBinaryPackageSpec {
    binary: TestBinarySpec,
}

#[derive(Debug, Clone, Deserialize)]
struct TestBinarySpec {
    version: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TestSourcePackageSpec {
    source: TestSourceSpec,
}

#[derive(Debug, Clone, Deserialize)]
struct TestSourceSpec {
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Hash, Eq, PartialEq)]
enum TestTargetSelector {
    Unix,
    Linux,
    Win,
    MacOs,
    Platform(String),
}

/// Converts a TestProjectModel into a ProjectModelV1
fn convert_test_model_to_project_model_v1(
    test_model: TestProjectModel,
) -> ProjectModelV1 {
    use std::str::FromStr;

    // Convert the targets
    let targets_v1 = TargetsV1 {
        default_target: Some(convert_target_to_v1(&test_model.targets.default_target)),
        targets: Some(
            test_model
                .targets
                .targets
                .into_iter()
                .map(|(selector, target)| {
                    (
                        convert_target_selector_to_v1(selector),
                        convert_target_to_v1(&target),
                    )
                })
                .collect(),
        ),
    };

    ProjectModelV1 {
        name: Some(test_model.name),
        version: Some(Version::from_str(&test_model.version).unwrap()),
        description: test_model.description,
        authors: test_model.authors,
        license: test_model.license,
        license_file: test_model.license_file.map(PathBuf::from),
        readme: test_model.readme.map(PathBuf::from),
        homepage: test_model.homepage.and_then(|h| url::Url::parse(&h).ok()),
        repository: test_model.repository.and_then(|r| url::Url::parse(&r).ok()),
        documentation: test_model
            .documentation
            .and_then(|d| url::Url::parse(&d).ok()),
        targets: Some(targets_v1),
    }
}

/// Converts a test Target to TargetV1
fn convert_target_to_v1(target: &TestTarget) -> TargetV1 {
    TargetV1 {
        build_dependencies: Some(
            target
                .build_dependencies
                .iter()
                .map(|(name, spec)| (name.clone(), convert_package_spec_to_v1(spec)))
                .collect(),
        ),
        host_dependencies: Some(
            target
                .host_dependencies
                .iter()
                .map(|(name, spec)| (name.clone(), convert_package_spec_to_v1(spec)))
                .collect(),
        ),
        run_dependencies: Some(
            target
                .run_dependencies
                .iter()
                .map(|(name, spec)| (name.clone(), convert_package_spec_to_v1(spec)))
                .collect(),
        ),
    }
}

/// Converts a test TargetSelector to TargetSelectorV1
fn convert_target_selector_to_v1(selector: TestTargetSelector) -> TargetSelectorV1 {
    match selector {
        TestTargetSelector::Unix => TargetSelectorV1::Unix,
        TestTargetSelector::Linux => TargetSelectorV1::Linux,
        TestTargetSelector::Win => TargetSelectorV1::Win,
        TestTargetSelector::MacOs => TargetSelectorV1::MacOs,
        TestTargetSelector::Platform(p) => TargetSelectorV1::Platform(p),
    }
}

/// Converts a test PackageSpec to PackageSpecV1
fn convert_package_spec_to_v1(spec: &TestPackageSpec) -> PackageSpecV1 {
    match spec {
        TestPackageSpec::Binary(binary_spec) => {
            let version_spec =
                VersionSpec::from_str(&binary_spec.binary.version, ParseStrictness::Lenient)
                    .unwrap_or(VersionSpec::Any);

            PackageSpecV1::Binary(Box::new(BinaryPackageSpecV1 {
                version: Some(version_spec),
                build: None,
                build_number: None,
                file_name: None,
                channel: None,
                subdir: None,
                md5: None,
                sha256: None,
                url: None,
                license: None,
            }))
        }
        TestPackageSpec::Source(source_spec) => {
            let inside_source = source_spec.source.clone();
            if let Some(path) = inside_source.path {
                let source_package_spec = SourcePackageSpecV1::Path(PathSpecV1 { path });
                PackageSpecV1::Source(source_package_spec)
            } else {
                unimplemented!("Only path source specs are supported for now");
            }
        }
    }
}
