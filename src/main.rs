use atom_syndication::{Content, Entry, FeedBuilder, Link, Text};
use chrono::DateTime;
use graphql_client::{GraphQLQuery, Response};

type Datetime = String;
type URL = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query.graphql",
    response_derives = "Debug"
)]
pub struct ProgramSet;

async fn perform_my_query(
    variables: program_set::Variables,
) -> anyhow::Result<program_set::ResponseData> {
    // this is the important line
    let request_body = ProgramSet::build_query(variables);

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.ardaudiothek.de/graphql")
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<program_set::ResponseData> = res.json().await?;
    // println!("{:#?}", response_body);

    response_body
        .data
        .ok_or(anyhow::anyhow!("Failed to fetch program"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let program_set = perform_my_query(program_set::Variables {
        id: "12642475".to_string(),
    })
    .await?
    .program_set
    .unwrap();

    let entries: Vec<Entry> = program_set
        .items
        .nodes
        .iter()
        .map(|item| {
            let audio = item.audios.as_ref().map(|audios| audios.first()).unwrap();
            let link = audio
                .map(|audio| {
                    vec![Link {
                        rel: "enclosure".to_string(),
                        title: Some(item.title.clone()),
                        mime_type: Some(audio.mime_type.clone()),
                        href: audio.url.clone(),
                        ..Default::default()
                    }]
                })
                .unwrap_or(Vec::new());

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
                links: link,
                published: Some(published),
                ..Entry::default()
            }
        })
        .collect();

    let atom_feed = FeedBuilder::default()
        .title(program_set.title)
        .entries(entries)
        .id(program_set.id)
        .build();

    let file = std::fs::File::create("atom.xml")?;

    atom_feed.write_to(file).unwrap();

    Ok(())
}
