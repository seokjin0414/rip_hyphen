use fantoccini::{Client, Locator};
use tokio;
use std::process::Command;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://localhost:4444";
    //
    let mut user_id: String = "".to_string();
    let mut user_pw: String = "".to_string();
    let mut user_number: String = "".to_string();

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

    // 3초 동안 대기(로딩...)
    println!("Waiting for 3 SEC...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 요금 조회 버튼 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_layout_wq_uuid_884")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the cost_view button: {}", e);
        } else {
            println!("cost_view clicked successfully");
        }
    } else {
        eprintln!("Failed to find the cost_view element");
    }

    /*// menu button 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_header_gnb_btnSiteMap")).await {
        eprintln!("Failed to find the menu button: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // 3초 동안 대기(로딩...)
    println!("Waiting for 5 SEC...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

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

    // cost_view 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_header_gnb_btn_selectFee")).await {
        eprintln!("Failed to find the cost_view: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // cost_view 클릭
    if let Ok(element) = client.find(Locator::Id("mf_wfm_header_gnb_btn_selectFee")).await {
        if let Err(e) = element.click().await {
            eprintln!("Failed to click the cost_view: {}", e);
        } else {
            println!("cost_view clicked successfully");
        }
    } else {
        eprintln!("Failed to find the cost_view:");
    }*/

    // 필드 로드 대기
    if let Err(e) = client.wait_for_find(Locator::Id("mf_wfm_layout_inp_searchCustNo")).await {
        eprintln!("Failed to find the user_number input: {}", e);
        client.close().await?;
        chromedriver_process.kill().expect("failed to kill ChromeDriver");
        return Err(Box::new(e) as Box<dyn Error>);
    }

    // 3초 동안 대기(로딩...)
    println!("Waiting for 3 SEC...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

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

    // 1초 동안 대기(로딩...)
    println!("Waiting for 3 SEC...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

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

    if option_selected {
        println!("'1년' option is selected successfully");
    } else {
        eprintln!("Failed to select '1년' option");
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