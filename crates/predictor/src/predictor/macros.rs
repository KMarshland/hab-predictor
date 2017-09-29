
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

/*
 * let a = result_or_return_why!(b, "Because b")
 *
 * -------
 *
 * let a = match b {
 *      Ok(b) => {
 *          b
 *      },
 *      Err(why) => {
 *          return Err(String::from("Because b"))
 *      }
 * }
 */
macro_rules! result_or_return_why {
    ($variable:expr, $why:expr) => {
        match $variable {
            Ok(value) => value,
            Err(_) => {
                return Err(String::from($why))
            }
        }
    };
}


/*
 * let a = some_or_return_why!(b, "Because b")
 *
 * -------
 *
 * let a = match b {
 *      Some(b) => {
 *          b
 *      },
 *      None => {
 *          return Err(String::from("Because b"))
 *      }
 * }
 */
macro_rules! some_or_return_why {
    ($variable:expr, $why:expr) => {
        match $variable {
            Some(value) => value,
            None => {
                return Err(String::from($why))
            }
        }
    };
}

macro_rules! return_error {
    ($why:expr) => {
        return Err(String::from($why));
    };
}
