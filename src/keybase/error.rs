error_chain! {
	foreign_links {
			Parsing(::serde_json::error::Error);
			IOErr(::std::io::Error);
			UTF8Err(std::string::FromUtf8Error);
	}
}