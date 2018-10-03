extern crate chrono;
extern crate codealong;
extern crate git2;
extern crate rs_es;
#[macro_use]
extern crate serde_json;

mod serialize;

use chrono::prelude::*;
use chrono::DateTime;
use codealong::walk;
use git2::Repository;
use rs_es::Client;

use serialize::serialize;

pub fn index(repo: &Repository) {
    let mut client = Client::new("http://localhost:9200").unwrap();
    for result in walk(&repo) {
        let commit = result.unwrap();
        let es_index = get_es_index(&commit.authored_at);
        let serialized = serialize(&commit);
        let mut index_op = client.index(&es_index, "commit");
        index_op
            .with_id(&commit.id)
            .with_doc(&serialized)
            .send()
            .expect("able to index");
        println!("put: {:#?} into {:#?}", serialized, es_index);
    }
}

fn get_es_index(date: &DateTime<Utc>) -> String {
    date.format("logstash-%Y.%m.%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn it_indexes() {
        let repo = Repository::open(Path::new("./fixtures/outreach")).unwrap();
        index(&repo);
    }
}
