
/*
 * let a = result_or_return!(b)
 *
 * -------
 *
 * let a = match b {
 *      Ok(b) => {
 *          b
 *      },
 *      Err(why) => {
 *          return Err(why)
 *      }
 * }
 */
macro_rules! result_or_return {
    ($variable:expr) => {
        match $variable {
            Ok(value) => value,
            Err(why) => {
                return Err(why)
            }
        }
    };
}

/*
 * let a = unsafe_dereference!(b)
 *
 * -------
 *
 * let a = unsafe {
 *      let ref mut node = *b;
 *      node
 * }
 */
macro_rules! unsafe_dereference {
    ($variable:expr) => {
        unsafe {
            let ref mut deref = *$variable;
            deref
        }
    };
}
