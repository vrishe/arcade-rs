
macro_rules! tryo {
	( $expr : expr ) => {
		match $expr {
			Some(value) => value,
			_ => return None
		} 
	}
}