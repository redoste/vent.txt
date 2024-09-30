function submit() {
	// These shenanigans are required because the browser can make weird things with <br/> and <div>
	// Using a <pre> as the parent node makes it a *bit* easier
	let message = "";
	document.querySelector("#form > .message").childNodes.forEach((e) => {
		if (e.tagName == "BR") {
			message += "\n";
		} else {
			message += e.textContent;
		}
	});
	message = message.trim();
	if (message === undefined || message.length <= 0) {
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
	let original_message = message ? message : original.querySelector(".message").childNodes[0].textContent;
	document.querySelector("#form > .message").innerText = original_message;
	document.querySelector("#form > .id").innerText = id;
}

function remove(id) {
	edit(id, "[removed]");
}
