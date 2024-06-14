use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, NaiveDate};
use dashmap::DashMap;
use dotenv::dotenv;
use fantoccini::{elements::Element, Client, ClientBuilder, Locator};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    env,
    process::{Child, Command},
    sync::Arc,
};
use futures::TryFutureExt;
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct PpData {
    claim_date: NaiveDate,
    usage: f64,
    paid: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let url = "http://localhost:4444";
    let user_id = env::var("PP_ID").expect("");
    let user_pw = env::var("PP_PW").expect("");
    let user_num = env::var("PP_NUMBER").expect("");

    // driver path
    let chromedriver_path = "/opt/homebrew/bin/chromedriver";

    // driver 실행
    let mut chromedriver_process = Command::new(chromedriver_path)
        .arg("--port=4444")
        .spawn()
        .expect("failed to start ChromeDriver");

    // driver 대기
    tokio::time::sleep(Duration::from_secs(2)).await;

    // headless, disable-gpu option
    // "--headless", "--disable-gpu"
    let capabilities: Map<String, Value> = serde_json::from_value(json!({
        "goog:chromeOptions": {
            "args": []
        }
    }))?;

    let client = loop {
        match ClientBuilder::native()
            .capabilities(capabilities.clone())
            .connect(url)
            .await
        {
            Ok(client) => break client,
            Err(e) => {
                eprintln!("Retrying to connect to WebDriver: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };
    let client_arc = Arc::new(client);

    // view size
    client_arc.set_window_rect(0, 0, 774, 857).await?;
    // 페이지 이동
    client_arc
        .goto("https://pp.kepco.co.kr/intro.do")
        .await
        .context("Failed to navigate")?;

    //공지 팝업 비활성화
    //notice_auto_popup

    // id 입력 로드 대기
    wait_for_element(
        &client_arc,
        Locator::Id("RSA_USER_ID"),
        &mut chromedriver_process,
    )
        .await?;
    // id 입력
    enter_value_in_element(
        &client_arc,
        Locator::Id("RSA_USER_ID"),
        &user_id,
    )
        .await?;
    // pw 입력
    enter_value_in_element(
        &client_arc,
        Locator::Id("RSA_USER_PWD"),
        &user_pw,
    )
        .await?;
    // 로그인 버튼 클릭
    click_element(
        &client_arc,
        Locator::XPath("/html/body/div[1]/div[2]/div[1]/form/fieldset/input[1]"),
    )
        .await?;

    // 로딩 대기
    wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(10),
    )
        .await?;
    
    // 월별 청구 요금 버튼 클릭
    // 해당 href 값으로 이동
    click_element(
        &client_arc,
        Locator::XPath("/html/body/div[1]/div[2]/div[1]/ul[4]/li[5]/a"),
    )
        .await?;

    // 뒤로 가기
    client_arc.goto("").await.context("Failed to navigate back")?;

    // 로딩 대기
    wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(10),
    )
        .await?;

    // data from parent_id -> vec
    let mut data_vec = parse_data_from_table(&client_arc, "//*[@id='grid']/tbody").await?;
    data_vec.sort_by(|a, b| b.claim_date.cmp(&a.claim_date));

    // '1년' 옵션을 선택
    //click_element_with_retries(&client_arc, Locator::XPath("//option[text()='1년']"), 10).await?;

    let reference_date = data_vec[data_vec.len() - 1]
        .claim_date
        .map(|date| format!("{}년 {:02}월", date.year(), date.month()))
        .unwrap_or_else(|| "N/A".to_string());


    // select 로드 대기
    wait_for_element(
        &client_arc,
        Locator::Id("year"),
        &mut chromedriver_process,
    )
        .await?;


    // select 에서 reference_date 옵션의 인덱스 search
    let select_locator = Locator::Id("year");
    let mut option_index = get_option_index(&client_arc, select_locator, &reference_date)
        .await
        .context("Failed to find option index")?;
    option_index+=1;

    // 1year over data parsing
    let mut additional_data_vec = parsing_options_data(
        &client_arc,
        select_locator,
        &option_index,
        &mut chromedriver_process,
    )
        .await?;

    // data 병합
    data_vec.append(&mut additional_data_vec);

    // JSON으로 변환
    let json_data =
        serde_json::to_string_pretty(&data_vec).context("Failed to serialize data to JSON")?;

    println!("{}", json_data);

    // 2분 동안 대기
    println!("Waiting for 2 minutes...");
    tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;

    // ChromeDriver 프로세스 종료
    chromedriver_process
        .kill()
        .expect("failed to kill ChromeDriver");

    Ok(())
}

// 요소 대기
async fn wait_for_element(
    client: &Client,
    locator: Locator<'_>,
    chromedriver_process: &mut Child,
) -> Result<Option<Element>> {
    match client.wait().for_element(locator).await {
        Ok(element) => Ok(Some(element)),
        Err(e) => {
            eprintln!("Failed to find the element: {:?}\n {}", locator, e);
            client
                .clone()
                .close()
                .await
                .context("Failed to close client")?;
            chromedriver_process
                .kill()
                .expect("failed to kill ChromeDriver");
            Err(anyhow::anyhow!("Failed to find the element: {:?}", e))
        }
    }
}

// 요소 클릭
async fn click_element(client: &Client, locator: Locator<'_>) -> Result<()> {
    if let Ok(element) = client.find(locator).await {
        element
            .click()
            .await
            .context(format!("Failed to click the element: {:?}", locator))?;
        println!("Element clicked successfully: {:?}", locator);
    } else {
        eprintln!("Failed to find the element: {:?}", locator);
        return Err(anyhow::anyhow!("Failed to find the element: {:?}", locator));
    }
    Ok(())
}

// 요소 클릭 반복
async fn click_element_with_retries(
    client: &Client,
    locator: Locator<'_>,
    max_attempts: u32,
) -> std::result::Result<(), anyhow> {
    let mut attempts = 0;
    loop {
        if attempts >= max_attempts {
            return Err(anyhow::anyhow!(
                "Failed to click the element after {} attempts",
                max_attempts
            ));
        }
        match client.find(locator).await {
            Ok(element) => match element.click().await {
                Ok(_) => {
                    println!(
                        "Element clicked successfully after {} attempts",
                        attempts + 1
                    );
                    return Ok(());
                }
                Err(e) => {
                    eprintln!(
                        "Failed to click the element (attempt {}): {}",
                        attempts + 1,
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    attempts += 1;
                }
            },
            Err(e) => {
                eprintln!(
                    "Retrying to find the element (attempt {}): {}",
                    attempts + 1,
                    e
                );
                // 요소를 찾지 못하면 잠시 대기 후 다시 시도
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
    }
}

// 요소에 값 입력
async fn enter_value_in_element(client: &Client, locator: Locator<'_>, text: &str) -> Result<()> {
    if let Ok(element) = client.find(locator).await {
        if let Err(e) = element.send_keys(text).await {
            eprintln!("Failed to enter text: {}", e);
        } else {
            println!("Text entered successfully: {:?}", locator);
        }
    } else {
        eprintln!("Failed to find the input element: {:?}", locator);
    }
    Ok(())
}

// 요소 비활성화 대기
async fn wait_for_element_display_none (
    client: &Client,
    locator: Locator<'_>,
    chromedriver_process: &mut Child,
    duration: Duration,
) -> Result<()> {
    let element = match wait_for_element(client, locator, chromedriver_process).await? {
        Some(element) => element,
        None => return Err(anyhow::anyhow!("Failed to find the element: {:?}", locator)),
    };

    let element_hidden = timeout(duration, async {
        loop {
            match element.attr("style").await {
                Ok(Some(style)) if style.contains("display: none") => {
                    println!("Element is hidden (style=\"display: none\")");
                    break;
                }
                Ok(_) => {
                    eprintln!("Element is not hidden, retrying...");
                }
                Err(e) => {
                    eprintln!("Failed to get style attribute: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
        .await;

    if element_hidden.is_err() {
        Err(anyhow::anyhow!(
            "Failed to find the element within the given duration"
        ))
    } else {
        Ok(())
    }
}

// select 요소에서 옵션 인덱스 찾기
async fn get_option_index(
    client: &Client,
    select_locator: Locator<'_>,
    text: &str,
) -> Result<usize> {
    let element = client
        .find(select_locator)
        .await
        .context("Failed to find select element")?;

    let options = element.find_all(Locator::XPath(".//option")).await?;
    for (index, option) in options.iter().enumerate() {
        if let Ok(option_text) = option.text().await {
            if option_text == text {
                return Ok(index);
            }
        }
    }
    Err(anyhow::anyhow!("Option with text '{}' not found", text))
}

// 자식 요소들의 ID -> DashMap
async fn get_children_ids_to_map(
    client: &Client,
    parent_xpath: &str,
) -> Result<Arc<DashMap<String, ()>>> {
    let script = format!(
        r#"
        let parent = document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
        if (parent === null) {{
            throw new Error("Parent element not found");
        }}
        let children = parent.children;
        let ids = [];
        for (let i = 0; i < children.length; i++) {{
            ids.push(children[i].id);
        }}
        return ids;
        "#,
        parent_xpath
    );

    let result = client
        .execute(&script, vec![])
        .await
        .context("Failed to execute script to get children IDs")?;

    let ids: Vec<String> = result
        .as_array()
        .context("Expected an array from the script result")?
        .iter()
        .map(|v| {
            v.as_str()
                .context("Expected a string in the array")
                .map(|s| s.to_string())
        })
        .collect::<Result<Vec<String>>>()?;

    let map = Arc::new(DashMap::new());
    for id in ids {
        map.insert(id, ());
    }

    Ok(map)
}

// get text from locator
async fn get_text_by_locator(client: &Client, locator: Locator<'_>) -> Option<String> {
    match client.find(locator).await.ok() {
        Some(element) => element.text().await.ok(),
        None => None,
    }
}

// get text from locator art index
async fn get_text_by_locator_at_index(
    client: &Client,
    locator: Locator<'_>,
    index: usize,
) -> Option<String> {
    match client.find_all(locator).await.ok() {
        Some(elements) => {
            if let Some(element) = elements.get(index) {
                return element.text().await.ok();
            }
            None
        }
        None => None,
    }
}

// parsing 청구 기간
fn parse_date(date_str: &str) -> Result<NaiveDate> {
    // 일자를 1로 설정
    let date_with_day = format!("{}.01일", date_str);
    NaiveDate::parse_from_str(&date_with_day, "%Y년 %m월 %d일").context("Failed to parse date")
}

// parsing 사용량
fn parse_use_kwh(kwh_str: &str) -> Result<f64> {
    let cleaned_str = kwh_str.replace(",", "").replace("kWh", "");
    cleaned_str
        .parse::<f64>()
        .context("Failed to parse use kWh")
}

// parsing 요금
fn parse_paid(amount_str: &str) -> Result<i64> {
    let amount_part = amount_str.split('원').next().unwrap_or(amount_str);

    let amount = amount_part.replace(",", "").replace(".", "");
    amount.parse::<i64>().context("Failed to parse amount")
}

// get_and_parsing_data year
async fn extract_data_year(client: &Client, parent_id: &str) -> Result<PpData> {
    let claim_date_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']/td[1]/a/span",
            parent_id
        )),
    )
    .await;

    let usage_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']/td[4]",
            parent_id
        )),
    )
    .await;

    let paid_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']/td[8]",
            parent_id
        )),
    )
    .await;

    let claim_date = claim_date_row.map_or(Ok(Default::default()), |date| parse_date(&date))?;
    let usage = usage_row.map_or(Ok(0.0), |kwh| parse_use_kwh(&kwh))?;
    let paid = paid_row.map_or(Ok(0), |paid| parse_paid(&paid))?;


    Ok(PpData {
        claim_date,
        usage,
        paid,
    })
}

// parse_data_from_parent_ids
async fn parse_data_from_table(
    client: &Arc<Client>,
    parent_xpath: &str,
) -> Result<Vec<PpData>> {
    let mut tasks = vec![];

    let map = get_children_ids_to_map(&client, parent_xpath).await?;

    for entry in map.iter() {
        let id = entry.key().clone();
        let client = Arc::clone(&client);
        let task = tokio::spawn(async move { extract_data_year(&client, &id).await });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    let mut data_vec = Vec::new();
    for result in results {
        match result {
            Ok(Ok(data)) => data_vec.push(data),
            Ok(Err(e)) => eprintln!("Failed to extract data: {}", e),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }

    Ok(data_vec)
}

// options 들의 결과값 parsing
async fn parsing_options_data(
    client: &Arc<Client>,
    select_locator: Locator<'_>,
    option_index: &usize,
    chromedriver_process: &mut Child,
) -> Result<Vec<PpData>> {
    // option 요소
    let options = client
        .find(select_locator)
        .await
        .context("Failed to find select element")?
        .find_all(Locator::Css("option"))
        .await
        .context("Failed to find options")?;

    let mut vec: Vec<PpData> = Vec::with_capacity(options.len() * 12);

    // option_index to last index data parsing
    for i in *option_index..options.len() {
        // 옵션 선택
        options[i]
            .click()
            .await
            .context("Failed to select option")?;

        // 조회 버튼 클릭
        // /html/body/div[2]/div[3]/div[2]/p/span[1]/a
        click_element(&client, Locator::XPath("//*[@id='txt']/div[2]/p/span[1]/a")).await?;

        // 로딩 대기
        wait_for_element_display_none(
            &client,
            Locator::Id("backgroundLayer"),
            chromedriver_process,
            Duration::from_secs(10),
        )
            .await?;

        // data parsing
        let mut data = parse_data_from_table(&client, "//*[@id='grid']/tbody").await?;
        vec.append(& mut data);
    }

    Ok(vec)
}
