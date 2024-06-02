use fantoccini::{Client, Locator};
use tokio;
use std::process::{Command, Child};
use std::error::Error;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::time::{timeout, Duration};


// 요소 대기
async fn wait_for_element(client: &Client, locator: Locator<'_>, chromedriver_process: &mut Child) -> Result<(), Box<dyn Error>> {
    if let Err(e) = client.wait().for_element(locator).await {
        eprintln!("Failed to find the element: {:?}\n {}", locator, e);
        client.clone().close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }
    Ok(())
}

// 요소 클릭
async fn click_element(client: &Client, locator: Locator<'_>) -> Result<(), Box<dyn Error>> {
    if let Ok(element) = client.find(locator).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the element: {:?}\n {}", locator, e);
        } else {
            println!("Element clicked successfully: {:?}", locator);
        }
    } else {
        eprintln!("Failed to find the element: {:?}", locator);
    }
    Ok(())
}

// 요소에 값 입력
async fn enter_value_in_element(client: &Client, locator: Locator<'_>, text: &str) -> Result<(), Box<dyn Error>> {
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
async fn wait_for_element_hidden(client: &Client, locator: Locator<'_>, duration: Duration) -> Result<(), Box<dyn Error>> {
    let element_hidden = timeout(duration, async {
        loop {
            match client.find(locator).await {
                Ok(element) => {
                    match element.attr("aria-hidden").await {
                        Ok(Some(value)) if value == "true" => {
                            println!("Element is hidden (aria-hidden=\"true\")");
                            break;
                        }
                        Ok(_) => {
                            eprintln!("Element is not hidden, retrying...");
                        }
                        Err(e) => {
                            eprintln!("Failed to get aria-hidden attribute: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Retrying to find the element: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }).await;

    if element_hidden.is_err() {
        Err(Box::new(element_hidden.err().unwrap()))
    } else {
        Ok(())
    }
}

// 요소 클릭 반복
async fn click_element_with_retries(client: &Client, locator: Locator<'_>, max_attempts: u32) -> Result<bool, Box<dyn Error>> {
    let mut attempts = 0;
    loop {
        if attempts >= max_attempts {
            return Ok(false); // 최대 시도 횟수에 도달하면 false 반환
        }
        match client.find(locator).await {
            Ok(element) => {
                match element.click().await {
                    Ok(_) => {
                        println!("Element clicked successfully");
                        return Ok(true);
                    }
                    Err(e) => {
                        eprintln!("Failed to click the element (attempt {}): {}", attempts + 1, e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        attempts += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("Retrying to find the element (attempt {}): {}", attempts + 1, e);
                // 요소를 찾지 못하면 잠시 대기 후 다시 시도
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let url = "http://localhost:4444";
    let user_id = env::var("USER_ID").expect("");
    let user_pw = env::var("USER_PW").expect("");
    let user_number = env::var("USER_NUMBER").expect("");

    // ChromeDriver 실행 경로 (Homebrew로 설치된 경로)
    let chromedriver_path = "/opt/homebrew/bin/chromedriver"; // 또는 설치된 ChromeDriver의 경로

    // ChromeDriver 실행
    let mut chromedriver_process = Command::new(chromedriver_path)
        .arg("--port=4444") // 포트를 4444로 설정
        .spawn()
        .expect("failed to start ChromeDriver");

    // 대기 ChromeDriver 완전 시작 대기
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // WebDriver 서버에 연결
    let client = loop {
        match Client::new(url).await {
            Ok(client) => break client,
            Err(e) => {
                eprintln!("Retrying to connect to WebDriver: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    };

    // 브라우저 창 크기 설정
    client.set_window_rect(0, 0, 774, 857).await?;

    // 페이지 이동
    if let Err(e) = client.goto("https://online.kepco.co.kr").await {
        eprintln!("Failed to navigate: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // menu button 로드 대기
    wait_for_element(&client, Locator::Id("mf_wfm_header_gnb_btnSiteMap"), &mut chromedriver_process).await?;
    // menu button 클릭
    click_element(&client, Locator::Id("mf_wfm_header_gnb_btnSiteMap")).await?;

    // login form 로드 대기
    wait_for_element(&client, Locator::Id("mf_wfm_header_gnb_mobileGoLogin"), &mut chromedriver_process).await?;
    // login form 클릭
    click_element(&client, Locator::Id("mf_wfm_header_gnb_mobileGoLogin")).await?;

    // id 입력 로드 대기
    wait_for_element(&client, Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_id"), &mut chromedriver_process).await?;
    // id 입력
    enter_value_in_element(&client, Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_id"), &user_id).await?;
    // pw 입력
    enter_value_in_element(&client, Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_pw"), &user_pw).await?;

    // 로그인 버튼 클릭
    click_element(&client, Locator::Id("mf_wfm_header_gnb_login_popup_wframe_btn_login")).await?;

    // /html/body/div[2]/div[3]/div/div/div[4]/div/div[2]/div[1]/a[3]
    // 요금 조회 버튼 로드 대기
    wait_for_element(&client, Locator::XPath("/html/body/div[2]/div[3]/div/div/div[4]/div/div[2]/div[1]/a[3]"), &mut chromedriver_process).await?;

    // 요금 조회 버튼 클릭 반복 시도
    let cost_view_clicked = click_element_with_retries(
        &client,
        Locator::XPath("/html/body/div[2]/div[3]/div/div/div[4]/div/div[2]/div[1]/a[3]"),
        10
    ).await?;

    // 필드 로드 대기
    wait_for_element(&client, Locator::Id("mf_wfm_layout_inp_searchCustNo"), &mut chromedriver_process).await?;

    // 1초 동안 대기(로딩...)
    //println!("Waiting for 1 SEC...");
    //tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // 로딩 대기
    wait_for_element_hidden(&client, Locator::Id("mf_wq_uuid_1_wq_processMsgComp"), Duration::from_secs(20)).await?;

    // 사용자 번호 입력
    enter_value_in_element(&client, Locator::Id("mf_wfm_layout_inp_searchCustNo"), &user_number).await?;
    // 사용자 번호 입력 후 검색 버튼 클릭
    click_element(&client, Locator::Id("mf_wfm_layout_btn_search")).await?;

    // 상세 요금 버튼 로드 대기
    wait_for_element(&client, Locator::Id("mf_wfm_layout_ui_generator_0_btn_moveDetail"), &mut chromedriver_process).await?;
    // 창에서 스크롤을 강제로 맨 아래로 내리기
    client.execute("window.scrollTo(0, document.body.scrollHeight);", vec![]).await?;

    // 상세 요금 버튼 클릭
    click_element(&client, Locator::Id("mf_wfm_layout_ui_generator_0_btn_moveDetail")).await?;

    // '1년' 옵션이 선택될 때까지 대기
    println!("Waiting for '1년' option to be selected...");
    let select_element = client.wait().for_element(Locator::Id("mf_wfm_layout_slb_searchMonth_input_0")).await?;

    // '1년' 옵션을 선택
    println!("Waiting for '1년' option to be selected...");
    let option_selected = loop {
        match select_element.find(Locator::XPath("//option[text()='1년']")).await {
            Ok(option) => {
                if let Err(e) = option.click().await {
                    eprintln!("Failed to click the '1년' option: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                } else {
                    println!("'1년' option clicked successfully");
                    break true;
                }
            },
            Err(e) => {
                eprintln!("Retrying to find the '1년' option: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    };

    // 로딩 대기
    wait_for_element_hidden(&client, Locator::Id("mf_wq_uuid_1_wq_processMsgComp"), Duration::from_secs(20)).await?;



    // 자식 요소들의 ID를 가져오기
    let result = client.execute(
        r#"
        let children = document.getElementById('mf_wfm_layout_ui_generator').children;
        let ids = [];
        for (let i = 0; i < children.length; i++) {
            ids.push(children[i].id);
        }
        return ids;
        "#,
        vec![]
    ).await?;

    // 결과를 벡터로 변환
    let ids: Vec<String> = result.as_array()
        .expect("Expected an array")
        .iter()
        .map(|v| v.as_str().expect("Expected a string").to_string())
        .collect();

    // DashMap에 ID들을 저장
    let map = Arc::new(DashMap::new());
    for id in ids {
        map.insert(id.clone(), ());
    }

    // DashMap의 내용을 출력
    for id in map.iter() {
        println!("ID: {}", id.key());
    }



    // 2분 동안 대기
    println!("Waiting for 2 minutes...");
    tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;

    // 브라우저 닫기
    if let Err(e) = client.close().await {
        eprintln!("Failed to close the browser: {}", e);
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // ChromeDriver 프로세스 종료
    chromedriver_process.kill().expect("failed to kill ChromeDriver");

    Ok(())
}