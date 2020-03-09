/*
const { JSDOM } = require('jsdom')
const fs = require('fs')

const data = String(fs.readFileSync('data.html'))
const dom = new JSDOM(data)
const document = dom.window.document
*/

class LmaoBGD {
    questionIds = []
    questionsMap = {}
    answersOfQuestion = {}
    answersMap = {}
    unknownQuestions = {}

    serverPayload() {
        const unknownQuestions = {}
        for (const key in this.unknownQuestions) {
            unknownQuestions[key] = {
                answers: this.answersOfQuestion[key].map(Number),
                answerUsed: Number(this.unknownQuestions[key]),
            }
        }
        return {
            questionMap: this.questionsMap,
            answerMap: this.answersMap,
            unknownQuestions,
        }
    }

    constructor(data, isInBrowser=false) {
        if (data == null) data = {}
        this.isInBrowser = isInBrowser
        this.answersData = data
    }

    runScrape() {
        const allQElems = document.getElementsByClassName("question-box")
        for (const elem of allQElems) {
            const attrs = elem.attributes
            let id
            for (const attr of attrs) {
                if (attr.name == "data-id") {
                    this.questionIds.push(attr.value)
                    id = attr.value
                    this.questionsMap[id] = elem.innerText
                }
            }
            const inputs = elem.querySelectorAll('input[type="radio"]')
            let list = []
            for (const input of inputs) {
                list.push(input.value)
                this.answersMap[input.value] = input.parentNode.parentNode.innerText
            }
            this.answersOfQuestion[id] = list
        }
        for (const key in this.answersOfQuestion) {
            console.log(`${key}: ${this.answersOfQuestion[key]}`)
        }
        // run this in browser
        // jsdom not supported
        if (this.isInBrowser) {
            console.log('Answers:')
            for (const key in this.answersMap) {
                console.log(`${key}: ${this.answersMap[key]}`)
            }
            console.log('Questions:')
            for (const key in this.questionsMap) {
                console.log(`${key}: ${this.questionsMap[key]}`)
            }
        }
    }

    fillAnswer() {
        for (const key in this.answersOfQuestion) {
            const text = this.questionsMap[key]
            console.log(`Answering ${key} (${text})`)
            const allAnswers = this.answersOfQuestion[key]
            let currentAnswer = this.answersData[key]
            if (currentAnswer == null) {
                const idx = Math.floor(Math.random() * 4)
                currentAnswer = allAnswers[idx]
                this.unknownQuestions[key] = currentAnswer
            }
            const radio = document.querySelector(`.question-box[data-id="${key}"] input[value="${currentAnswer}"]`)
            radio.click()
            console.log(`Clicking answer ${radio.value} (${this.answersMap[radio.value]})`)
        }
        console.log('Unknown questions and guessed answer:')
        for (const key in this.unknownQuestions) {
            const answer = this.unknownQuestions[key]
            const text = this.answersMap[answer]
            console.log(`${key} (${this.answersMap[key]}): ${answer} (${text})`)
        }
    }

}
//module.exports = { LmaoBGD }
