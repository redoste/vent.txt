function get_text_content(element) {
	let out = "";
	element.childNodes.forEach((e) => {
		if (e.tagName == "BR") {
			out += "\n";
		} else {
			out += e.textContent;
		}
	});
	return out;
}

function submit() {
	// These shenanigans are required because the browser can make weird things with <br/> and <div>
	// Using a <pre> as the parent node makes it a *bit* easier
	let message = get_text_content(document.querySelector("#form > .message")).trim();
	if (message.length <= 0) {
		return;
	}

	let id = document.querySelector("#form > .id").innerText;

	let form = document.querySelector("#form > form");
	form.querySelector("[name=message]").value = message;
	form.querySelector("[name=id]").value = id;
	form.submit();
}

function reset() {
	document.querySelector("#form > .message").innerText = "";
	document.querySelector("#form > .id").innerText = "+";
}

function edit(id, message) {
	let original = document.getElementById(id);
	if (!message) {
		message = get_text_content(original.querySelector(".message"));
	}
	document.querySelector("#form > .message").innerText = message;
	document.querySelector("#form > .id").innerText = id;
}

function remove(id) {
	edit(id, "[removed]");
}
