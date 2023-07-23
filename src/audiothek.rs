use atom_syndication::{Content, Entry, Feed, FeedBuilder, Link, Text};
use chrono::DateTime;
use graphql_client::{GraphQLQuery, Response};
use program_set::{ProgramSetProgramSetItemsNodes, ProgramSetProgramSetItemsNodesAudios};

type Datetime = String;
#[allow(clippy::upper_case_acronyms)]
type URL = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query.graphql",
    response_derives = "Debug"
)]
pub struct ProgramSet;

/// Load the data from the Audiothek, then build the atom feed
pub async fn fetch_feed(variables: program_set::Variables) -> anyhow::Result<Feed> {
    let request_body = ProgramSet::build_query(variables);

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.ardaudiothek.de/graphql")
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<program_set::ResponseData> = res.json().await?;
    // println!("{:#?}", response_body);

    let program_set = response_body
        .data
        .ok_or(anyhow::anyhow!("Failed to fetch program"))?
        .program_set
        .unwrap();

    let entries: Vec<Entry> = program_set
        .items
        .nodes
        .iter()
        .map(node_to_episode)
        .collect();

    let modified = DateTime::parse_from_rfc3339(&program_set.last_item_modified.unwrap()).unwrap();
    let mut atom_feed = FeedBuilder::default()
        .title(program_set.title)
        .entries(entries)
        .id(program_set.id)
        .updated(modified)
        .build();

    atom_feed.logo = program_set
        .image
        .as_ref()
        .and_then(|image| image.url.clone())
        .map(|url| url.replace("{width}", "512"));

    atom_feed.icon = program_set
        .image
        .and_then(|image| image.url)
        .map(|url| url.replace("{width}", "255"));

    Ok(atom_feed)
}

fn audio_to_link(audio: &ProgramSetProgramSetItemsNodesAudios) -> Link {
    Link {
        href: audio.url.clone(),
        rel: "enclosure".to_string(),
        mime_type: Some(audio.mime_type.clone()),
        title: audio.title.clone(),
        ..Default::default()
    }
}

fn node_to_episode(item: &ProgramSetProgramSetItemsNodes) -> Entry {
    let links = item
        .audios
        .as_ref()
        .map(|audios| audios.iter().map(audio_to_link))
        .unwrap()
        .collect();

    let summary = item.summary.clone();
    let published = DateTime::parse_from_rfc3339(&item.publish_date).unwrap();

    let content = Content {
        content_type: Some("text".to_string()),
        value: Some(item.summary.clone().unwrap()),
        ..Content::default()
    };

    Entry {
        id: item.id.clone(),
        title: Text::plain(item.title.clone()),
        summary: Some(Text::plain(summary.unwrap())),
        content: Some(content),
        links,
        published: Some(published),
        updated: published,
        ..Entry::default()
    }
}
