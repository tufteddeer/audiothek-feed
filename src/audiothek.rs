use atom_syndication::{Content, Entry, Feed, Link, Text};
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
        .and_then(|data| data.program_set)
        .ok_or(anyhow::anyhow!("Failed to fetch program"))?;

    let entries: Vec<Entry> = program_set
        .items
        .nodes
        .iter()
        .map(node_to_episode)
        .collect();

    let mut atom_feed = Feed {
        title: program_set.title.into(),
        entries,
        id: program_set.id,
        ..Default::default()
    };

    if let Some(Ok(updated)) = program_set
        .last_item_modified
        .map(|date| DateTime::parse_from_rfc3339(&date))
    {
        atom_feed.updated = updated;
    }

    atom_feed.logo = program_set
        .image
        .as_ref()
        .and_then(|image| image.url.clone())
        .map(|url| image_url(&url, 512));

    atom_feed.icon = program_set
        .image
        .as_ref()
        .and_then(|image| image.url.clone())
        .map(|url| image_url(&url, 256));

    Ok(atom_feed)
}

fn image_url(url: &str, width: u32) -> String {
    url.replace("{width}", format!("{width}").as_str())
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
    let published = DateTime::parse_from_rfc3339(&item.publish_date).ok();

    let content = Content {
        content_type: Some("text".to_string()),
        value: item.summary.clone(),
        ..Content::default()
    };

    let mut entry = Entry {
        id: item.id.clone(),
        title: Text::plain(item.title.clone()),
        summary: summary.map(Text::plain),
        content: Some(content),
        links,
        published,
        ..Entry::default()
    };

    if let Some(published) = published {
        entry.updated = published;
    }

    entry
}
