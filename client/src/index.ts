export class LmaoBGD {
  private questionIds: number[] = [];
  private questionsMap: Map<number, string> = new Map();
  private answersOfQuestion: Map<number, number[]> = new Map();
  private answersMap: Map<number, string> = new Map();
  private unknownQuestions: Map<number, number> = new Map();
  private groupText = "";
  private groupId = 0;

  private serverPayload() {
    const payload: any = {
      questionMap: {},
      answerMap: {},
      unknownQuestions: {},
      groupText: this.groupText,
      group: this.groupId
    };
    for (const key of this.unknownQuestions.keys()) {
      payload.unknownQuestions[key] = {
        answers: this.answersOfQuestion.get(key),
        answerUsed: this.unknownQuestions.get(key)
      };
    }
    for (const key of this.questionsMap.keys()) {
      payload.questionMap[key] = this.questionsMap.get(key);
    }
    for (const key of this.answersMap.keys()) {
      payload.answerMap[key] = this.answersMap.get(key);
    }
    return payload;
  }

  public async upload(url = "http://localhost:5000/api/upload", auth: string) {
    const encoded = btoa(auth + ":");
    try {
      const resp = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "text/json",
          Authorization: "Basic " + encoded
        },
        body: JSON.stringify(this.serverPayload())
      });
      if (resp.ok) {
        console.log("Upload complete");
      } else {
        console.error(`Error ${resp.status} ${resp.statusText} ${resp.body}`);
      }
    } catch (e) {
      console.error(e);
    }
  }

  public async getData(url = "http://localhost:5000/api/data", auth: string) {
    const encoded = btoa(auth + ":");
    try {
      const resp = await fetch(url, {
        headers: {
          Authorization: "Basic " + encoded
        }
      });
      if (resp.ok) {
        const json: any = await resp.json();
        for (const key in json) {
          this.answersData.set(parseInt(key), json[key]);
        }
      } else {
        console.error(`Error ${resp.status} ${resp.statusText} ${resp.body}`);
      }
    } catch (e) {
      console.error(e);
    }
  }

  public static async run(api = "http://localhost:5000/api", key: string) {
    const lmao = new LmaoBGD();
    lmao.runScrape();
    await lmao.getData(`${api}/data`, key);
    lmao.fillAnswer();
    await lmao.upload(`${api}/upload`, key);
  }

  public constructor(
    private answersData = new Map<number, number>(),
    private isInBrowser = false
  ) {}

  public runScrape() {
    const titleSelector = "body .row .col-12 h1";
    const idSelector = "body .row .row .col-12 div";
    const titleElem = document.querySelector(
      titleSelector
    ) as HTMLHeadingElement;
    this.groupText = titleElem.innerText;
    console.log(`Test name: ${this.groupText}`);
    let idStr = (document.querySelector(
      idSelector
    ) as HTMLDivElement).innerText.split(":");
    idStr.reverse();
    this.groupId = Number(idStr[0].trim());
    console.log(`Code: ${this.groupId}`);
    const allQuestionElems = document.getElementsByClassName(
      "question-box"
    ) as HTMLCollectionOf<HTMLElement>;
    for (const elem of allQuestionElems) {
      const attrs = elem.attributes;
      const idAttr = attrs.getNamedItem("data-id");
      if (idAttr != null) {
        let id = parseInt(idAttr.value);
        this.questionsMap.set(id, elem.innerText);
        this.questionIds.push(id);
        const inputs = elem.querySelectorAll(
          'input[type="radio"]'
        ) as NodeListOf<HTMLInputElement>;
        let list = [];
        for (const input of inputs) {
          const aid = parseInt(input.value);
          list.push(aid);
          let cur: HTMLElement = input;
          for (let i = 0; i < 2 && cur.parentNode != null; i++) {
            cur = cur.parentNode as HTMLElement;
          }
          this.answersMap.set(aid, cur.innerText);
        }
        this.answersOfQuestion.set(id, list);
      }
    }
    for (const key of this.answersOfQuestion.keys()) {
      console.log(`${key}: ${this.answersOfQuestion.get(key)}`);
    }
    // run this in browser
    // jsdom not supported
    if (this.isInBrowser) {
      console.log("Answers:");
      for (const key of this.answersMap.keys()) {
        console.log(`${key}: ${this.answersMap.get(key)}`);
      }
      console.log("Questions:");
      for (const key of this.questionsMap.keys()) {
        console.log(`${key}: ${this.questionsMap.get(key)}`);
      }
    }
  }

  public fillAnswer() {
    for (const key of this.answersOfQuestion.keys()) {
      const text = this.questionsMap.get(key);
      console.log(`Answering ${key} (${text})`);
      const allAnswers = this.answersOfQuestion.get(key);
      if (allAnswers == null) continue;
      let currentAnswer = this.answersData.get(key);
      if (currentAnswer == null) {
        const idx = Math.floor(Math.random() * allAnswers.length);
        currentAnswer = allAnswers[idx];
        this.unknownQuestions.set(key, currentAnswer);
      }
      const radio = document.querySelector(
        `.question-box[data-id="${key}"] input[value="${currentAnswer}"]`
      ) as HTMLInputElement;
      radio.click();
      console.log(
        `Clicking answer ${radio.value} (${this.answersMap.get(currentAnswer)})`
      );
    }
    console.log("Unknown questions and guessed answer:");
    for (const key of this.unknownQuestions.keys()) {
      const answer = this.unknownQuestions.get(key);
      if (answer == null) continue;
      const text = this.answersMap.get(answer);
      console.log(
        `${key} (${this.questionsMap.get(key)}): ${answer} (${text})`
      );
    }
  }
}

export default LmaoBGD;
