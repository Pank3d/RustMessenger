use {crate::utils, std::path::PathBuf, tokio::fs};

pub(crate) type ReadDataTaskError = std::io::Error;

pub(crate) async fn run(
	basedir: PathBuf,
	hash: impl AsRef<[u8]>,
) -> Result<Vec<u8>, ReadDataTaskError> {
	let hash = hex::encode(hash.as_ref());
	Ok(fs::read(
		utils::data_path(basedir, hash.clone())
			.join(hash)
			.with_extension("dat"),
	)
	.await?)
}
