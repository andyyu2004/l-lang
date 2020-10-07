#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(or_patterns)]

#[macro_use]
extern crate log;

mod autoderef;
mod check;
mod expr;
mod pat;
mod stmt;
mod tyconv;
mod type_of;

use autoderef::Autoderef;
use check::FnCtx;
use tyconv::TyConv;
use type_of::Typeof;
