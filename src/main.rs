use fantoccini::{Client, Locator};
use tokio;
use std::process::Command;
use std::error::Error;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://localhost:4444";
    let mut attempts = 0;
    let max_attempts = 10;

    dotenv().ok();
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

    // 대기 ChromeDriver 완전 시작 기다림
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
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_header_gnb_btnSiteMap")).await {
        eprintln!("Failed to find the menu button: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // mf_wfm_header_gnb_mobileGoLogin
    // menu button 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_header_gnb_btnSiteMap")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the menu button: {}", e);
        } else {
            println!("menu button clicked successfully");
        }
    } else {
        eprintln!("Failed to find the menu button:");
    }

    // login form 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_header_gnb_mobileGoLogin")).await {
        eprintln!("Failed to find the login form: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // mf_wfm_header_gnb_mobileGoLogin
    // login form 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_header_gnb_mobileGoLogin")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the login form: {}", e);
        } else {
            println!("login form clicked successfully");
        }
    } else {
        eprintln!("Failed to find the login form:");
    }

    // id 입력 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_id")).await {
        eprintln!("Failed to find the login ID input: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // id 입력
    if let Ok(element) = client.find(Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_id")).await {
        if let Err(e) = element.send_keys(&user_id).await {
            eprintln!("Failed to enter user ID: {}", e);
        } else {
            println!("User ID entered successfully");
        }
    } else {
        eprintln!("Failed to find the user ID input element");
    }

    // pw 입력
    if let Ok(element) = client.find(Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_pw")).await {
        if let Err(e) = element.send_keys(&user_pw).await {
            eprintln!("Failed to enter password: {}", e);
        } else {
            println!("Password entered successfully");
        }
    } else {
        eprintln!("Failed to find the password input element");
    }

    // 로그인 버튼 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_header_gnb_login_popup_wframe_btn_login")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the login button: {}", e);
        } else {
            println!("Login button clicked successfully");
        }
    } else {
        eprintln!("Failed to find the login button element");
    }

    // 요금 조회 버튼 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_layout_wq_uuid_884")).await {
        eprintln!("Failed to find the cost_view: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    let cost_view_clicked = loop {
        if attempts >= max_attempts {
            break false; // 최대 시도 횟수에 도달하면 루프 종료
        }
        match client.find(Locator::Id("mf_wfm_layout_wq_uuid_884")).await {
            Ok(element) => {
                match element.click().await {
                    Ok(_) => {
                        println!("Cost view clicked successfully");
                        break true; // 성공적으로 클릭하면 루프 종료
                    }
                    Err(e) => {
                        eprintln!("Failed to click the cost_view button (attempt {}): {}", attempts + 1, e);
                        // 클릭 실패 시 잠시 대기 후 다시 시도
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        attempts += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("Retrying to find the cost_view element (attempt {}): {}", attempts + 1, e);
                // 요소를 찾지 못하면 잠시 대기 후 다시 시도
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
    };

    /*// 요금 조회 버튼 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_layout_wq_uuid_884")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the cost_view button: {}", e);
        } else {
            println!("cost_view clicked successfully");
        }
    } else {
        eprintln!("Failed to find the cost_view element");
    }*/

    // 필드 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_layout_inp_searchCustNo")).await {
        eprintln!("Failed to find the user_number input: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // 1초 동안 대기(로딩...)
    println!("Waiting for 1 SEC...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // 로딩 확인
    let element_hidden = timeout(Duration::from_secs(20), async {
        loop {
            match client.find(Locator::Id("mf_wq_uuid_1_wq_processMsgComp")).await {
                Ok(element) => {
                    match element.attr("aria-hidden").await {
                        Ok(Some(value)) if value == "true" => {
                            println!("Element is hidden (aria-hidden=\"true\")");
                            break;
                        }
                        Ok(_) => {
                            eprintln!("Element is not hidden yet, retrying...");
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


    // 사용자 번호 입력
    if let Ok(element) = client.find(Locator::Id("mf_wfm_layout_inp_searchCustNo")).await {
        if let Err(e) = element.send_keys(&user_number).await {
            eprintln!("Failed to enter user_number: {}", e);
        } else {
            println!("user_number entered successfully");
        }
    } else {
        eprintln!("Failed to find the user_number input element");
    }

    // 사용자 번호 입력 후 검색 버튼 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_layout_btn_search")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the search_button: {}", e);
        } else {
            println!("search_button clicked successfully");
        }
    } else {
        eprintln!("Failed to find the search_button");
    }

    // mf_wfm_layout_ui_generator_0_btn_moveDetail
    // 상세 요금 버튼 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_layout_ui_generator_0_btn_moveDetail")).await {
        eprintln!("Failed to find the move_detail : {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // 1초 동안 대기
    println!("Waiting for 1 SEC...");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // 창에서 스크롤을 강제로 맨 아래로 내리기
    client.execute("window.scrollTo(0, document.body.scrollHeight);", vec![]).await?;

    // 상세 요금 버튼 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_layout_ui_generator_0_btn_moveDetail")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the move_detail: {}", e);
        } else {
            println!("move_detail clicked successfully");
        }
    } else {
        eprintln!("Failed to find the move_detail");
    }

    // '1년' 옵션이 선택될 때까지 대기
    println!("Waiting for '1년' option to be selected...");
    let select_element = client.wait_for_find(Locator::Id("mf_wfm_layout_slb_searchMonth_input_0")).await?;

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
    let element_hidden = timeout(Duration::from_secs(20), async {
        loop {
            match client.find(Locator::Id("mf_wq_uuid_1_wq_processMsgComp")).await {
                Ok(element) => {
                    match element.attr("aria-hidden").await {
                        Ok(Some(value)) if value == "true" => {
                            println!("Element is hidden (aria-hidden=\"true\")");
                            break;
                        }
                        Ok(_) => {
                            eprintln!("Element is not hidden yet, retrying...");
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

    // JavaScript를 사용하여 특정 요소의 자식 요소들의 ID를 가져오기
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


    // 결과를 정수로 변환
    let div_count = result.as_i64().unwrap_or(0);

    println!("Number of child divs: {}", div_count);



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