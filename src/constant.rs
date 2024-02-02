macro_rules! TABLE_ROW {
    () => {
        "<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>
                <button class=\"btn btn-danger\"
                hx-get=\"/expenses/{}/edit\"
                hx-trigger=\"edit\"
                _=\"on click
                   if .editing is not empty
                       Swal.fire({{title: 'Already Editing',
                                  showCancelButton: true,
                                  confirmButtonText: 'Yep, Edit This Row!',
                                  text:'Hey!  You are already editing a row!  Do you want to cancel that edit and continue?'}})
                       if the result's isConfirmed is false
                         halt
                       end
                       send cancel to .editing
                     end
                     trigger edit\">
                    Edit
                </button>
            </td>
        </tr>"
    };
}

macro_rules! VALID_EMAIL {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"email\">Email</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"email\"
              name=\"email\"
              placeholder=\"email@server.com\"
              aria-label=\"Email\"
              aria-invalid=\"false\"
              autocomplete=\"email\"
              hx-post=\"/validate/email\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
        </div>"
    };
}

macro_rules! INVALID_EMAIL {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"email\">Email</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"email\"
              name=\"email\"
              placeholder=\"email@server.com\"
              aria-label=\"Email\"
              aria-invalid=\"true\"
              autocomplete=\"email\"
              hx-post=\"/validate/email\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">Please enter a valid email address, such as someone@gmail.com.</div>
        </div>"
    };
}

//macro_rules! EMAIL_TAKEN {
//    () => {
//        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
//            <div class=\"grid\">
//            <label for=\"email\">Email</label>
//            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
//            </div>
//            <input
//              type=\"email\"
//              name=\"email\"
//              placeholder=\"email@server.com\"
//              aria-label=\"Email\"
//              aria-invalid=\"true\"
//              autocomplete=\"email\"
//              hx-post=\"/validate/email\"
//              hx-sync=\"closest form:abort\"
//              hx-indicator=\"#ind\"
//              value=\"{}\"
//              required
//            />
//            <div class='error-message' style=\"color:red;\">That email is already taken.  Please enter another email address.</div>
//        </div>"
//    };
//}

macro_rules! VALID_USERNAME {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"username\">Username</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"text\"
              name=\"username\"
              placeholder=\"Username\"
              aria-label=\"Login\"
              aria-invalid=\"false\"
              autocomplete=\"nickname\"
              pattern=\"[0-9a-z]{{3,20}}\"
              title=\"3 to 20 characters, lowercase letters or numbers only\"
              hx-post=\"/validate/username\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
        </div>"
    };
}

macro_rules! INVALID_USERNAME {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"username\">Username</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"text\"
              name=\"username\"
              placeholder=\"Username\"
              aria-label=\"Login\"
              aria-invalid=\"true\"
              autocomplete=\"nickname\"
              pattern=\"[0-9a-z]{{3,20}}\"
              title=\"3 to 20 characters, lowercase letters or numbers only\"
              hx-post=\"/validate/username\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">Username should be 3 to 20 characters long and only consist of lowercase letters or numbers.  Please enter another username.</div>
        </div>"
    };
}

macro_rules! MATCHING_PASSWORDS {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"confirm_password\">Confirm Password</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"password\"
              name=\"confirm_password\"
              placeholder=\"Password\"
              aria-label=\"Password\"
              aria-invalid=\"false\"
              id=\"confirm_password\"
              hx-post=\"/validate/passwords\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              hx-include=\"#password\"
              value=\"{}\"
              required
            />
        </div>"
    };
}

macro_rules! MISMATCHING_PASSWORDS {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"confirm_password\">Confirm Password</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"password\"
              name=\"confirm_password\"
              placeholder=\"Password\"
              aria-label=\"Password\"
              aria-invalid=\"true\"
              id=\"confirm_password\"
              hx-post=\"/validate/passwords\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              hx-include=\"#password\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">Passwords don't match.</div>
        </div>"
    };
}

macro_rules! MATCHING_NEW_PASSWORDS {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <input
              type=\"password\"
              name=\"confirm_password\"
              placeholder=\"Confirm new password\"
              autocomplete=\"new-password\"
              aria-label=\"Password\"
              aria-invalid=\"false\"
              id=\"new_password2\"
              hx-post=\"/validate/new-passwords\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              hx-include=\"#new_password\"
              value=\"{}\"
              required
            />
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
        </div>"
    };
}

macro_rules! MISMATCHING_NEW_PASSWORDS {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <input
              type=\"password\"
              name=\"confirm_password\"
              placeholder=\"Confirm new password\"
              autocomplete=\"new-password\"
              aria-label=\"Password\"
              aria-invalid=\"true\"
              id=\"new_password2\"
              hx-post=\"/validate/new-passwords\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              hx-include=\"#new_password\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">Passwords don't match.</div>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
        </div>"
    };
}

macro_rules! STRONG_PASSWORD {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"password\">Password</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
                type=\"password\"
                name=\"password\"
                placeholder=\"Password\"
                aria-label=\"Password\"
                aria-invalid=\"false\"
                autocomplete=\"new-password\"
                id=\"password\"
                hx-post=\"/validate/password-strength\"
                hx-sync=\"closest form:abort\"
                hx-indicator=\"#ind\"
                value=\"{}\"
                required
            />
        </div>"
    };
}

macro_rules! WEAK_PASSWORD {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"password\">Password</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
                type=\"password\"
                name=\"password\"
                placeholder=\"Password\"
                aria-label=\"Password\"
                aria-invalid=\"true\"
                autocomplete=\"new-password\"
                id=\"password\"
                hx-post=\"/validate/password-strength\"
                hx-sync=\"closest form:abort\"
                hx-indicator=\"#ind\"
                value=\"{}\"
                required
            />
            <div class='error-message' style=\"color:red;\">Password too weak. Try using a password generator or choosing a password with uppercase, lowercase, numbers and special characters.</div>
        </div>"
    };
}

macro_rules! STRONG_NEW_PASSWORD {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <input
                type=\"password\"
                name=\"password\"
                placeholder=\"New password\"
                aria-label=\"Password\"
                aria-invalid=\"false\"
                autocomplete=\"new-password\"
                id=\"new_password\"
                hx-post=\"/validate/new-password-strength\"
                hx-sync=\"closest form:abort\"
                hx-indicator=\"#ind\"
                value=\"{}\"
                required
            />
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
        </div>"
    };
}

macro_rules! WEAK_NEW_PASSWORD {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <input
                type=\"password\"
                name=\"password\"
                placeholder=\"New password\"
                aria-label=\"Password\"
                aria-invalid=\"true\"
                autocomplete=\"new-password\"
                id=\"new_password\"
                hx-post=\"/validate/new-password-strength\"
                hx-sync=\"closest form:abort\"
                hx-indicator=\"#ind\"
                value=\"{}\"
                required
            />
            <div class='error-message' style=\"color:red;\">Password too weak. Try using a password generator or choosing a password with uppercase, lowercase, numbers and special characters.</div>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
        </div>"
    };
}

macro_rules! DELETE_EXPENSE_MODAL {
    () => {
        "<dialog id=\"delete-expense-modal\" open>
            <article>
                <a href=\"#close\"
                  aria-label=\"Close\"
                  class=\"close\"
                  _=\"on click trigger toggleModal\">
                </a>
                <h3>Delete the expense</h3>
                <p>Are you sure you want to delete this expense?</p>
                <footer>
                    <div class=\"grid\">
                        <button hx-delete=\"/expenses/{}\" hx-target=\"#delete-modal-here\" class=\"contrast\" hx-trigger=\"click\">Delete</button>
                        <button _=\"on click trigger toggleModal\" type=\"button\">Close</button>
                    </div>
                </footer>
            </article>
        </dialog>"
    };
}

pub(crate) use DELETE_EXPENSE_MODAL;
pub(crate) use INVALID_EMAIL;
pub(crate) use INVALID_USERNAME;
pub(crate) use MATCHING_NEW_PASSWORDS;
pub(crate) use MATCHING_PASSWORDS;
pub(crate) use MISMATCHING_NEW_PASSWORDS;
pub(crate) use MISMATCHING_PASSWORDS;
pub(crate) use STRONG_NEW_PASSWORD;
pub(crate) use STRONG_PASSWORD;
pub(crate) use TABLE_ROW;
pub(crate) use VALID_EMAIL;
pub(crate) use VALID_USERNAME;
pub(crate) use WEAK_NEW_PASSWORD;
pub(crate) use WEAK_PASSWORD;
